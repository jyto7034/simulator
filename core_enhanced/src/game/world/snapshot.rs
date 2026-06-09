use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{
    GameError, SkillCatalogCastTargetDto, SkillCatalogDeliveryKind, SkillCatalogDto,
    SkillCatalogSkillDto, SkillCatalogStepDto, SkillCatalogTileAreaDto,
};
use crate::game::combat_player_spawns::effective_combat_profile_for_employee;
use crate::game::data::ItemRef;
use crate::game::employee::ActiveConsumableModifier;
use crate::game::resources::{ActiveNodeContent, ConsumableItemDto, GameState};
use crate::game::reward::RewardOption;

#[derive(Debug, Serialize)]
struct ActiveConsumableModifierSnapshotDto<'a> {
    source_item_uuid: Uuid,
    definition_id: &'a str,
    name: &'a str,
    tier: crate::game::data::consumable_data::ConsumableTier,
    duration_policy: crate::game::data::consumable_data::ConsumableDurationPolicy,
    remaining_combat_nodes: u32,
    effect: &'a crate::game::data::consumable_data::ConsumableEffect,
}

impl<'a> From<&'a ActiveConsumableModifier> for ActiveConsumableModifierSnapshotDto<'a> {
    fn from(modifier: &'a ActiveConsumableModifier) -> Self {
        Self {
            source_item_uuid: modifier.source_item_uuid,
            definition_id: &modifier.definition_id,
            name: &modifier.name,
            tier: modifier.tier,
            duration_policy: modifier.duration_policy,
            remaining_combat_nodes: modifier.remaining_combat_nodes,
            effect: &modifier.effect,
        }
    }
}

fn snapshot_game_error_code(error: &GameError) -> &'static str {
    match error {
        GameError::EventNotFound => "event_not_found",
        GameError::EventTypeMismatch => "event_type_mismatch",
        GameError::InvalidAction => "invalid_action",
        GameError::NotInShopState => "not_in_shop_state",
        GameError::NotInRewardState => "not_in_reward_state",
        GameError::ShopRerollNotAllowed => "shop_reroll_not_allowed",
        GameError::ShopItemNotFound => "shop_item_not_found",
        GameError::InventoryFull => "inventory_full",
        GameError::InventoryItemNotFound => "inventory_item_not_found",
        GameError::AlreadyOwnedArtifact => "already_owned_artifact",
        GameError::InsufficientResources => "insufficient_resources",
        GameError::MissingResource(_) => "missing_resource",
        GameError::InvalidUnitStats(_) => "invalid_unit_stats",
        GameError::InvalidStaticData(_) => "invalid_static_data",
        GameError::NotImplemented(_) => "not_implemented",
        GameError::SkillFragmentIncompatible { .. } => "skill_fragment_incompatible",
        GameError::OutOfBounds => "out_of_bounds",
        GameError::PositionOccupied => "position_occupied",
        GameError::UnitAlreadyPlaced => "unit_already_placed",
        GameError::UnitNotFound => "unit_not_found",
    }
}

fn effective_profile_error_json(error: &GameError) -> Value {
    json!({
        "code": snapshot_game_error_code(error),
        "message": format!("{error:?}"),
    })
}

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
            "skill_catalog": serde_json::to_value(self.get_skill_catalog_snapshot_dto())
                .map_err(|error| GameError::InvalidStaticData(error.to_string()))?,
            "roster": self.get_employee_roster_snapshot_json()?,
            "roster_order": self.get_roster_order_snapshot_json()?,
            "inventory": self.get_inventory_snapshot_json()?,
            "resources": {
                "enkephalin": enkephalin,
                "qliphoth": qliphoth,
            },
        }))
    }

    fn game_state_context_json(&self) -> Value {
        match self.get_state() {
            GameState::NotStarted => json!({ "type": "not_started" }),
            GameState::SelectingStarterEmployees => json!({
                "type": "selecting_starter_employees",
                "required_count": super::RUN_SYSTEM_POLICY.setup.starter_employee_count,
                "candidates": self.state.starter_candidates,
            }),
            GameState::ViewingMap => json!({ "type": "viewing_map" }),
            GameState::NodeConfirm {
                node_id,
                kind_id,
                category,
            } => {
                let run = self.state.run.as_ref();
                let combat_preview = run.and_then(|run| run.combat_previews.get(&node_id));
                let abnormality_attempt = run
                    .filter(|_| matches!(category, crate::game::map::MapNodeCategory::Combat))
                    .map(|run| run.abnormality_attempt_dto(node_id));
                json!({
                    "type": "node_confirm",
                    "node_id": node_id,
                    "kind_id": kind_id,
                    "category": category,
                    "combat_preview": combat_preview,
                    "abnormality_attempt": abnormality_attempt,
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
            GameState::CombatResult { battle_uuid } => json!({
                "type": "combat_result",
                "battle_uuid": battle_uuid,
            }),
            GameState::InBattle { battle_uuid } => {
                let active = self.state.active_battle.as_ref();
                let abnormality_attempt = active.and_then(|battle| {
                    (battle.node_type != crate::game::combat_preview::CombatNodeType::Boss).then(
                        || {
                            self.state.run.as_ref().map(|run| {
                                run.abnormality_attempt_dto(crate::game::map::MapNodeId(
                                    battle.battle_uuid,
                                ))
                            })
                        },
                    )?
                });
                let can_retreat = active.is_some_and(|battle| {
                    battle.node_type != crate::game::combat_preview::CombatNodeType::Boss
                        && !battle.execution.is_finished()
                });
                json!({
                    "type": "in_battle",
                    "battle_uuid": battle_uuid,
                    "node_type": active.map(|battle| battle.node_type),
                    "mission_variant": active.map(|battle| battle.mission_variant),
                    "encounter_id": active.map(|battle| battle.encounter_id.as_str()),
                    "combat_preview": active.map(|battle| &battle.combat_preview),
                    "last_pushed_timeline_seq": active.and_then(|battle| battle.last_pushed_timeline_seq),
                    "deployment": active.and_then(|battle| battle.live_deployment_dto(&self.state.roster)),
                    "playback": active.map(|battle| battle.playback_state()),
                    "abnormality_attempt": abnormality_attempt,
                    "can_retreat": can_retreat,
                })
            }
            GameState::GameOver => json!({ "type": "game_over" }),
            GameState::RunComplete => json!({ "type": "run_complete" }),
            GameState::RunFailed { reason } => json!({
                "type": "run_failed",
                "reason": reason,
            }),
        }
    }

    fn get_skill_catalog_snapshot_dto(&self) -> SkillCatalogDto {
        let mut skills = self
            .game_data
            .skill_data
            .skills
            .iter()
            .map(|skill| {
                let cast_target = skill.cast_target_definition().map(
                    |(range_units, target, defense_tile_range, air_capable)| {
                        SkillCatalogCastTargetDto {
                            range_units,
                            target_policy: target.clone(),
                            defense_tile_range: defense_tile_range.cloned(),
                            air_capable,
                        }
                    },
                );
                SkillCatalogSkillDto {
                    skill_id: skill.id.clone(),
                    display_name: skill.name.clone(),
                    kind: skill.kind,
                    focus_time_ms: skill.focus_time_ms,
                    focus_permissions: skill.focus_permissions,
                    cast_target,
                    steps: skill
                        .steps
                        .iter()
                        .map(|step| {
                            let tile_area = match &step.delivery {
                                crate::game::ability::DeliveryDef::TileArea { area } => {
                                    Some(SkillCatalogTileAreaDto {
                                        anchor: area.anchor,
                                        tile_origin: area.tile_origin,
                                        tracking: area.tracking,
                                        hit_targets: area.hit_targets,
                                        include_caster: area.include_caster,
                                        tick_policy: area.tick_policy,
                                        duration_ms: area.duration_ms,
                                        tick_interval_ms: area.tick_interval_ms,
                                    })
                                }
                                _ => None,
                            };
                            SkillCatalogStepDto {
                                step_id: step.id.clone(),
                                delay_ms: step.delay_ms,
                                range_units: step.range_units,
                                target_policy: step.target.clone(),
                                defense_tile_range: step.defense_tile_range.clone(),
                                air_capable: step.air_capable,
                                delivery: match &step.delivery {
                                    crate::game::ability::DeliveryDef::Instant => {
                                        SkillCatalogDeliveryKind::Instant
                                    }
                                    crate::game::ability::DeliveryDef::Projectile { .. } => {
                                        SkillCatalogDeliveryKind::Projectile
                                    }
                                    crate::game::ability::DeliveryDef::TileArea { .. } => {
                                        SkillCatalogDeliveryKind::TileArea
                                    }
                                },
                                tile_area,
                                effects_count: step.effects.len(),
                                presentation: step.presentation.clone(),
                            }
                        })
                        .collect::<Vec<_>>(),
                }
            })
            .collect::<Vec<_>>();
        skills.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
        SkillCatalogDto {
            version: 1,
            range_source_of_truth: "defense_tile_range".to_string(),
            skills,
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
                    "requirements": metadata.map(|fragment| &fragment.compatibility),
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

        let mut consumables = inventory
            .consumables
            .iter()
            .map(|owned| {
                json!(crate::game::resources::ConsumableItemDto::from_owned_consumable(owned))
            })
            .collect::<Vec<_>>();
        consumables.sort_by(|left, right| left["uuid"].to_string().cmp(&right["uuid"].to_string()));

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
            "consumables": consumables,
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

    pub fn get_roster_order_snapshot_json(&self) -> Result<Value, GameError> {
        let roster_order = self.roster_order()?;

        let slots = roster_order
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
            "max_slots": roster_order.max_slots,
            "slots": slots,
        }))
    }

    pub fn get_employee_roster_snapshot_json(&self) -> Result<Value, GameError> {
        let roster = self.roster()?;
        let inventory = self.inventory()?;
        let roster_order = self.roster_order().ok();

        let mut employees = roster
            .iter()
            .map(|employee| -> Result<Value, GameError> {
                let roster_slot =
                    roster_order.and_then(|roster_order| roster_order.slot_of(employee.uuid));
                let effective_profile_result = effective_combat_profile_for_employee(
                    roster,
                    inventory,
                    &self.state.skill_fragments,
                    &self.game_data,
                    employee.uuid,
                );
                let (effective_profile, effective_profile_error) = match effective_profile_result {
                    Ok(profile) => (Some(profile), Value::Null),
                    Err(error) => (None, effective_profile_error_json(&error)),
                };
                let skill_fragment_compatibility = self
                    .state
                    .skill_fragments
                    .owned_ids()
                    .map(|fragment_id| {
                        let metadata = self.game_data.skill_fragment_data.get_by_id(fragment_id);
                        let report = self.skill_fragment_compatibility_report_for_employee(
                            employee.uuid,
                            fragment_id,
                        )?;
                        Ok(json!({
                            "id": fragment_id,
                            "name": metadata.map(|fragment| fragment.name.as_str()),
                            "is_compatible": report.is_compatible,
                            "failure_codes": report.failure_codes,
                            "requirements": metadata.map(|fragment| &fragment.compatibility),
                        }))
                    })
                    .collect::<Result<Vec<_>, GameError>>()?;
                Ok(json!({
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
                        "deployment_affinity": employee.combat_profile.battle_profile.deployment_affinity,
                        "effective_stats": effective_profile.as_ref().map(|profile| profile.stats),
                        "effective_basic_attack": effective_profile.as_ref().map(|profile| profile.basic_attack.clone()),
                        "effective_weapon_profile": effective_profile.as_ref().and_then(|profile| profile.weapon_profile.clone()),
                        "effective_skill_id": effective_profile.as_ref().and_then(|profile| profile.skill_id.clone()),
                        "effective_deployment_affinity": effective_profile.as_ref().map(|profile| profile.deployment_affinity),
                        "effective_profile_error": effective_profile_error,
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
                        "compatibility": skill_fragment_compatibility,
                    },
                    "active_consumable_modifier": employee
                        .active_consumable_modifier
                        .as_ref()
                        .map(ActiveConsumableModifierSnapshotDto::from),
                    "equipped_items": employee.loadout.item_slot.iter().map(|equipped| {
                        let equipment = inventory.equipments.get_item(&equipped.instance_uuid);
                        json!({
                            "instance_uuid": equipped.instance_uuid,
                            "base_uuid": equipped.base_uuid,
                            "equipment_type": equipped.equipment_type,
                            "definition_id": equipment.map(|item| item.meta.id.as_str()),
                            "name": equipment.map(|item| item.meta.name.as_str()),
                            "weapon_profile": equipment.and_then(|item| item.meta.weapon_profile.clone()),
                            "bound": equipment.map(|item| item.meta.bound).unwrap_or(false),
                            "can_unequip": equipment.map(|item| !item.meta.bound).unwrap_or(false),
                            "cannot_unequip_reason": equipment.and_then(|item| item.meta.bound.then(|| item.meta.cannot_unequip_reason.as_str())),
                        })
                    }).collect::<Vec<_>>(),
                    "roster_slot": roster_slot,
                }))
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        employees.sort_by(|left, right| left["uuid"].to_string().cmp(&right["uuid"].to_string()));

        Ok(json!({
            "employees": employees,
            "available_employee_ids": roster.available_employee_ids(),
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
            ItemRef::Consumable(meta) => {
                let mut value =
                    serde_json::to_value(ConsumableItemDto::from_metadata(item_uuid, meta))
                        .map_err(|error| GameError::InvalidStaticData(error.to_string()))?;
                if let Some(object) = value.as_object_mut() {
                    object.insert("kind".to_string(), json!("consumable"));
                    object.insert("id".to_string(), json!(meta.id));
                    object.insert("type".to_string(), json!("consumable"));
                }
                value
            }
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
        let Some(selected) = self.state.active_node_content.as_ref() else {
            return Ok(None);
        };

        let value = match selected {
            ActiveNodeContent::Shop(shop) => json!({
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
            ActiveNodeContent::Reward(reward) => json!({
                "type": "reward",
                "stage_uuid": reward.stage_uuid,
                "mode": reward.mode,
                "rewards": reward.rewards.iter().map(display_reward_option_snapshot_json).collect::<Vec<_>>(),
                "selected_reward_uuid": reward.selected_reward_uuid,
                "can_skip": reward.can_skip,
            }),
            ActiveNodeContent::Support(support) => json!({
                "type": "support",
                "node_id": support.node_id,
                "support_mode": support.support_mode,
                "support_type": support.support_type,
                "choices": support.choices,
                "selected_support_type": support.selected_support_type,
                "target_candidates": support.target_candidates,
                "selected_employee_uuid": support.selected_employee_uuid,
                "selected_medical_treatment": support.selected_medical_treatment,
            }),
            ActiveNodeContent::Maintenance(maintenance) => json!({
                "type": "maintenance",
                "node_id": maintenance.node_id,
                "maintenance_options": self.maintenance_options(),
            }),
            ActiveNodeContent::HeadquartersContact(headquarters) => json!({
                "type": "headquarters_contact",
                "node_id": headquarters.node_id,
                "options": headquarters.options,
                "recruitment_candidates": headquarters.recruitment_candidates,
                "shop_pool_id": headquarters.shop_pool_id,
            }),
            ActiveNodeContent::CombatBattle(battle) => json!({
                "type": "combat_battle",
                "abnormality_id": battle.abnormality_id,
                "encounter_id": battle.encounter_id,
                "node_type": battle.node_type,
                "mission_variant": battle.mission_variant,
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
