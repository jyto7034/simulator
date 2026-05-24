use serde_json::{json, Value};
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::GameError;
use crate::game::data::ItemRef;
use crate::game::resources::{GameState, SelectedEventState};
use crate::game::reward::RewardOption;

impl GameCore {
    pub fn get_run_snapshot_json(&self) -> Result<Value, GameError> {
        let map = self
            .state
            .run
            .as_ref()
            .map(|run| json!(run.map_progression.view(&run.map, &run.run_progression)))
            .unwrap_or(Value::Null);
        let run_progression = self
            .state
            .run
            .as_ref()
            .map(|state| {
                let run = &state.run_progression;
                json!({
                    "run_seed": run.run_seed,
                    "act_index": run.act_index,
                    "max_acts": run.max_acts,
                    "current_act_seed": run.current_act_seed(),
                })
            })
            .unwrap_or(Value::Null);
        let map_progression = self
            .state
            .run
            .as_ref()
            .map(|state| {
                let progression = &state.map_progression;
                json!({
                    "current_node_id": progression.current_node_id,
                    "available_node_ids": progression.available_node_ids,
                    "completed_node_ids": progression.completed_node_ids,
                })
            })
            .unwrap_or(Value::Null);
        let current_node_session = self
            .state
            .node_session
            .as_ref()
            .map(|session| json!(session))
            .unwrap_or(Value::Null);
        let enkephalin = self.state.enkephalin.amount;
        let recon_charge = self
            .state
            .run
            .as_ref()
            .map(|run| run.recon_charge)
            .unwrap_or(0);
        let qliphoth = json!({
            "amount": self.state.qliphoth.amount,
            "level": self.qliphoth_level_name(self.state.qliphoth.level),
        });
        Ok(json!({
            "game_state": self.game_state_name(),
            "game_state_context": self.game_state_context_json(),
            "allowed_actions": self.get_allowed_actions(),
            "run_progression": run_progression,
            "map_progression": map_progression,
            "map": map,
            "current_node_session": current_node_session,
            "selected_event": self.get_selected_event_snapshot_json()?,
            "roster": self.get_employee_roster_snapshot_json()?,
            "field": self.get_field_snapshot_json()?,
            "bench": self.get_bench_snapshot_json()?,
            "inventory": self.get_inventory_snapshot_json()?,
            "resources": {
                "enkephalin": enkephalin,
                "recon_charge": recon_charge,
                "qliphoth": qliphoth,
            },
        }))
    }

    fn game_state_context_json(&self) -> Value {
        match self.get_state() {
            GameState::NotStarted => json!({ "type": "not_started" }),
            GameState::SelectingStarterEmployees => json!({
                "type": "selecting_starter_employees",
                "required_count": super::RUN_SYSTEM_POLICY.starter_employee_count,
                "candidates": self.state.starter_candidates,
            }),
            GameState::ViewingMap => json!({ "type": "viewing_map" }),
            GameState::NodeConfirm {
                node_id,
                kind_id,
                category,
            } => {
                let combat_preview = self
                    .state
                    .run
                    .as_ref()
                    .and_then(|run| run.combat_previews.get(&node_id));
                let combat_deployment = self
                    .state
                    .run
                    .as_ref()
                    .and_then(|run| run.combat_deployments.get(&node_id));
                json!({
                    "type": "node_confirm",
                    "node_id": node_id,
                    "kind_id": kind_id,
                    "category": category,
                    "combat_preview": combat_preview,
                    "combat_deployment": combat_deployment,
                })
            }
            GameState::InNode {
                node_id,
                kind_id,
                category,
            } => json!({
                "type": "in_node",
                "node_id": node_id,
                "kind_id": kind_id,
                "category": category,
            }),
            GameState::InShop { shop_uuid } => json!({
                "type": "in_shop",
                "shop_uuid": shop_uuid,
            }),
            GameState::InReward { reward_uuid } => json!({
                "type": "in_reward",
                "reward_uuid": reward_uuid,
            }),
            GameState::InRewardClaimed { reward_uuid } => json!({
                "type": "in_reward_claimed",
                "reward_uuid": reward_uuid,
            }),
            GameState::InCombatReplay { battle_uuid } => json!({
                "type": "in_combat_replay",
                "battle_uuid": battle_uuid,
            }),
            GameState::InBattle { battle_uuid } => json!({
                "type": "in_battle",
                "battle_uuid": battle_uuid,
            }),
            GameState::GameOver => json!({ "type": "game_over" }),
            GameState::RunComplete => json!({ "type": "run_complete" }),
            GameState::RunFailed { reason } => json!({
                "type": "run_failed",
                "reason": reason,
            }),
        }
    }

    pub fn get_inventory_snapshot_json(&self) -> Result<Value, GameError> {
        let inventory = self.inventory()?;
        let skill_fragments = self
            .state
            .skill_fragments
            .owned_ids()
            .map(|fragment_id| {
                let metadata = self.game_data.skill_fragment_data.get_by_id(fragment_id);
                let progress = self.state.skill_fragments.progress(fragment_id);
                json!({
                    "id": fragment_id,
                    "count": self.state.skill_fragments.count(fragment_id),
                    "name": metadata.map(|fragment| fragment.name.as_str()),
                    "effect": metadata.map(|fragment| format!("{:?}", fragment.effect)),
                    "progress": {
                        "research_progress": progress.research_progress,
                        "research_completion_count": progress.research_completion_count,
                        "upgrade_level": progress.upgrade_level,
                        "awakening_progress": progress.awakening_progress,
                        "awakening_available": progress.awakening_available,
                        "awakened": progress.awakened,
                    },
                })
            })
            .collect::<Vec<_>>();
        let skill_fragment_progress = self
            .state
            .skill_fragments
            .progress_entries()
            .map(|(fragment_id, progress)| {
                let metadata = self.game_data.skill_fragment_data.get_by_id(fragment_id);
                json!({
                    "id": fragment_id,
                    "owned_count": self.state.skill_fragments.count(fragment_id),
                    "name": metadata.map(|fragment| fragment.name.as_str()),
                    "progress": {
                        "research_progress": progress.research_progress,
                        "research_completion_count": progress.research_completion_count,
                        "upgrade_level": progress.upgrade_level,
                        "awakening_progress": progress.awakening_progress,
                        "awakening_available": progress.awakening_available,
                        "awakened": progress.awakened,
                    },
                })
            })
            .collect::<Vec<_>>();

        let mut equipments = inventory
            .equipments
            .iter()
            .map(|owned| {
                json!({
                    "instance_uuid": owned.instance_uuid,
                    "item": crate::game::resources::EquipmentItemDto::from_owned_equipment(owned),
                    "equipped_to": owned.equipped_to,
                })
            })
            .collect::<Vec<_>>();
        equipments.sort_by(|left, right| {
            left["instance_uuid"]
                .to_string()
                .cmp(&right["instance_uuid"].to_string())
        });

        let mut artifacts = inventory
            .artifacts
            .iter()
            .map(|artifact| {
                json!(crate::game::resources::ArtifactItemDto::from_metadata(
                    artifact.as_ref()
                ))
            })
            .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| left["uuid"].to_string().cmp(&right["uuid"].to_string()));

        let mut equipment_materials = inventory
            .equipment_materials
            .iter()
            .map(|(material_id, amount)| {
                let metadata = self.game_data.equipment_data.get_material_by_id(material_id);
                json!({
                    "material_id": material_id,
                    "amount": amount,
                    "name": metadata.map(|material| material.name.as_str()),
                    "description": metadata.map(|material| material.description.as_str()),
                    "material_type": metadata.map(|material| format!("{:?}", material.material_type)),
                    "rarity": metadata.map(|material| material.rarity),
                    "equipment_type": metadata.and_then(|material| material.equipment_type),
                })
            })
            .collect::<Vec<_>>();
        equipment_materials.sort_by(|left, right| {
            left["material_id"]
                .to_string()
                .cmp(&right["material_id"].to_string())
        });

        Ok(json!({
            "equipments": equipments,
            "equipment_materials": equipment_materials,
            "artifacts": artifacts,
            "skill_fragments": skill_fragments,
            "skill_fragment_progress": skill_fragment_progress,
            "pending_research_deliveries": self.state.skill_fragments.pending_research_deliveries()
                .map(|(fragment_id, count)| json!({
                    "fragment_id": fragment_id,
                    "count": count,
                }))
                .collect::<Vec<_>>(),
            "fragment_dust": self.state.skill_fragments.fragment_dust(),
        }))
    }

    pub fn get_bench_snapshot_json(&self) -> Result<Value, GameError> {
        let bench = self.bench()?;

        let slots = bench
            .slots
            .iter()
            .enumerate()
            .map(|(slot, unit_uuid)| {
                json!({
                    "slot": slot,
                    "unit_uuid": unit_uuid,
                })
            })
            .collect::<Vec<_>>();

        Ok(json!({
            "max_slots": bench.max_slots,
            "slots": slots,
        }))
    }

    pub fn get_employee_roster_snapshot_json(&self) -> Result<Value, GameError> {
        let roster = self.roster()?;
        let bench = self.bench().ok();

        let mut employees = roster
            .iter()
            .map(|employee| {
                let bench_slot = bench.and_then(|bench| bench.slot_of(employee.uuid));
                json!({
                    "uuid": employee.uuid,
                    "name": employee.name,
                    "level": employee.level,
                    "experience": employee.experience,
                    "life_state": format!("{:?}", employee.life_state),
                    "availability": format!("{:?}", employee.availability),
                    "available_for_combat": employee.is_available_for_combat(),
                    "trauma": employee.trauma,
                    "health": {
                        "current_hp": employee.health.current_hp,
                        "max_hp": employee.health.max_hp,
                    },
                    "injuries": employee.injuries.iter().map(|injury| {
                        json!({
                            "id": injury.id,
                            "severity": injury.severity,
                        })
                    }).collect::<Vec<_>>(),
                    "trust": {
                        "score": employee.trust.score,
                        "band": format!("{:?}", employee.trust.band()),
                        "traits": employee.trust.traits.iter().map(|trait_kind| format!("{:?}", trait_kind)).collect::<Vec<_>>(),
                        "memories": employee.trust.memories.iter().map(|memory| {
                            json!({
                                "kind": format!("{:?}", memory.kind),
                                "intensity": memory.intensity,
                                "remaining_nodes": memory.remaining_nodes,
                            })
                        }).collect::<Vec<_>>(),
                        "recent_reactions": employee.trust.recent_reactions.iter().map(|reaction| {
                            json!({
                                "event": format!("{:?}", reaction.event),
                                "trust_delta": reaction.trust_delta,
                                "cue_count": reaction.cue_count,
                                "combat_modifier_count": reaction.combat_modifier_count,
                                "trauma_modifier_count": reaction.trauma_modifier_count,
                            })
                        }).collect::<Vec<_>>(),
                    },
                    "combat_profile": {
                        "grade": format!("{:?}", employee.combat_profile.grade),
                        "base_stats": employee.combat_profile.battle_profile.stats,
                        "basic_attack": employee.combat_profile.battle_profile.basic_attack,
                        "effective_stats": employee.combat_profile_for_battle(&self.game_data.skill_fragment_data, &self.state.skill_fragments).ok().map(|profile| profile.stats),
                        "effective_basic_attack": employee.combat_profile_for_battle(&self.game_data.skill_fragment_data, &self.state.skill_fragments).ok().map(|profile| profile.basic_attack),
                        "effective_skill_id": employee.combat_profile_for_battle(&self.game_data.skill_fragment_data, &self.state.skill_fragments).ok().and_then(|profile| profile.skill_id),
                    },
                    "skill_fragments": {
                        "equipped": employee.skill_fragments.equipped_ids().iter().map(|fragment_id| {
                            let metadata = self.game_data.skill_fragment_data.get_by_id(fragment_id);
                            json!({
                                "id": fragment_id,
                                "name": metadata.map(|fragment| fragment.name.as_str()),
                                "effect": metadata.map(|fragment| format!("{:?}", fragment.effect)),
                            })
                        }).collect::<Vec<_>>(),
                        "baseline_ids": employee.skill_fragments.baseline_ids(),
                        "active_fragment_id": employee.skill_fragments.active_fragment_id(),
                        "equipped_ids": employee.skill_fragments.equipped_ids(),
                    },
                    "equipped_items": employee.loadout.item_slot.iter().map(|equipped| {
                        json!({
                            "instance_uuid": equipped.instance_uuid,
                            "base_uuid": equipped.base_uuid,
                            "equipment_type": equipped.equipment_type,
                        })
                    }).collect::<Vec<_>>(),
                    "bench_slot": bench_slot,
                })
            })
            .collect::<Vec<_>>();
        employees.sort_by(|left, right| left["uuid"].to_string().cmp(&right["uuid"].to_string()));

        Ok(json!({
            "employees": employees,
            "available_employee_ids": roster.available_employee_ids(),
        }))
    }

    pub fn get_field_snapshot_json(&self) -> Result<Value, GameError> {
        let field = self.field()?;

        let mut placements = field
            .placements
            .iter()
            .map(|(position, placement)| {
                json!({
                    "position": position,
                    "unit_uuid": placement.uuid,
                    "side": placement.side,
                })
            })
            .collect::<Vec<_>>();
        placements.sort_by(|left, right| {
            let ly = left["position"]["y"].as_i64().unwrap_or_default();
            let ry = right["position"]["y"].as_i64().unwrap_or_default();
            let lx = left["position"]["x"].as_i64().unwrap_or_default();
            let rx = right["position"]["x"].as_i64().unwrap_or_default();
            ly.cmp(&ry).then_with(|| lx.cmp(&rx))
        });

        Ok(json!({
            "width": field.width,
            "height": field.height,
            "placements": placements,
        }))
    }

    fn display_items_snapshot_json(&self, item_uuids: &[Uuid]) -> Result<Vec<Value>, GameError> {
        item_uuids
            .iter()
            .copied()
            .map(|item_uuid| self.display_item_snapshot_json(item_uuid))
            .collect()
    }

    fn display_item_snapshot_json(&self, item_uuid: Uuid) -> Result<Value, GameError> {
        let item = self.game_data.item(&item_uuid).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "selected event references missing item uuid {item_uuid}"
            ))
        })?;

        let value = match item {
            ItemRef::Equipment(meta) => {
                let equipment_type = equipment_type_display_key(meta.equipment_type);
                json!({
                    "uuid": item_uuid,
                    "kind": "equipment",
                    "definition_id": meta.id,
                    "id": meta.id,
                    "name": meta.name,
                    "rarity": meta.rarity,
                    "price": meta.price,
                    "type": equipment_type,
                    "equipment_type": equipment_type,
                    "description": "",
                })
            }
            ItemRef::Artifact(meta) => json!({
                "uuid": item_uuid,
                "kind": "artifact",
                "definition_id": meta.id,
                "id": meta.id,
                "name": meta.name,
                "rarity": meta.rarity,
                "price": meta.price,
                "type": "artifact",
                "artifact_type": "artifact",
                "effect_id": meta.id,
                "description": meta.description,
            }),
            ItemRef::Abnormality(meta) => json!({
                "uuid": item_uuid,
                "kind": "abnormality",
                "definition_id": meta.id,
                "id": meta.id,
                "name": meta.name,
                "rarity": meta.risk_level,
                "risk_level": meta.risk_level,
                "price": meta.price,
                "type": "abnormality",
                "description": "",
            }),
        };

        Ok(value)
    }

    pub fn get_selected_event_snapshot_json(&self) -> Result<Option<Value>, GameError> {
        let Some(selected) = self.state.selected_event.as_ref() else {
            return Ok(None);
        };

        let value = match &selected.event {
            SelectedEventState::Shop(shop) => json!({
                "type": "shop",
                "id": shop.id,
                "name": shop.name,
                "uuid": shop.uuid,
                "shop_type": shop.shop_type,
                "can_reroll": shop.can_reroll,
                "visible_items": self.display_items_snapshot_json(&shop.visible_items)?,
                "hidden_items": self.display_items_snapshot_json(&shop.hidden_items)?,
                "visible_item_uuids": shop.visible_items,
                "hidden_item_uuids": shop.hidden_items,
            }),
            SelectedEventState::Reward(reward) => json!({
                "type": "reward",
                "stage_uuid": reward.stage_uuid,
                "mode": reward.mode,
                "rewards": reward.rewards.iter().map(display_reward_option_snapshot_json).collect::<Vec<_>>(),
                "selected_reward_uuid": reward.selected_reward_uuid,
                "can_skip": reward.can_skip,
            }),
            SelectedEventState::Support(support) => json!({
                "type": "support",
                "node_id": support.node_id,
                "support_mode": support.support_mode,
                "support_type": support.support_type,
                "choices": support.choices,
                "selected_support_type": support.selected_support_type,
                "target_candidates": support.target_candidates,
                "selected_employee_uuid": support.selected_employee_uuid,
                "selected_medical_treatment": support.selected_medical_treatment,
                "maintenance_options": self.maintenance_options_for_support(support),
            }),
            SelectedEventState::HeadquartersContact(headquarters) => json!({
                "type": "headquarters_contact",
                "node_id": headquarters.node_id,
                "options": headquarters.options,
                "recruitment_candidates": headquarters.recruitment_candidates,
                "shop_pool_id": headquarters.shop_pool_id,
            }),
            SelectedEventState::CombatBattle(battle) => json!({
                "type": "combat_battle",
                "abnormality_id": battle.abnormality_id,
                "encounter_id": battle.encounter_id,
                "node_type": battle.node_type,
                "abnormality_uuid": battle.abnormality_uuid,
                "winner": battle.winner,
                "reward_mode": battle.reward_mode,
                "rewards": battle.rewards.iter().map(display_reward_option_snapshot_json).collect::<Vec<_>>(),
                "has_timeline": true,
            }),
        };

        Ok(Some(value))
    }
}

fn equipment_type_display_key(
    equipment_type: crate::game::data::equipment_data::EquipmentType,
) -> &'static str {
    match equipment_type {
        crate::game::data::equipment_data::EquipmentType::Weapon => "weapon",
        crate::game::data::equipment_data::EquipmentType::Armor => "armor",
        crate::game::data::equipment_data::EquipmentType::Accessory => "accessory",
    }
}

fn display_reward_option_snapshot_json(reward: &RewardOption) -> Value {
    json!({
        "uuid": reward.uuid,
        "kind": "reward",
        "id": reward.id,
        "name": reward.name,
        "rarity": Value::Null,
        "price": 0,
        "description": reward.description,
        "icon": reward.icon,
        "tags": reward.tags,
        "effects": reward.effects,
    })
}
