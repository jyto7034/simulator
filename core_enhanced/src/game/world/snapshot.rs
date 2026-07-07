use serde_json::Value;
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{
    ActiveConsumableModifierSnapshotDto, DisplayArtifactItemSnapshotDto,
    DisplayConsumableItemSnapshotDto, DisplayEquipmentItemSnapshotDto, DisplayItemSnapshotDto,
    EmployeeCombatProfileSnapshotDto, EmployeeEquippedItemSnapshotDto, EmployeeHealthSnapshotDto,
    EmployeeInjurySnapshotDto, EmployeeRosterSnapshotDto, EmployeeSkillFragmentBriefSnapshotDto,
    EmployeeSkillFragmentCompatibilitySnapshotDto, EmployeeSkillFragmentsSnapshotDto,
    EmployeeSnapshotDto, EmployeeTrustMemorySnapshotDto, EmployeeTrustReactionSnapshotDto,
    EmployeeTrustSnapshotDto, GameError, GameStateContextDto,
    InventoryEquipmentMaterialSnapshotDto, InventoryEquipmentSnapshotDto,
    InventorySkillFragmentProgressSnapshotDto, InventorySkillFragmentSnapshotDto,
    InventorySnapshotDto, MapNavigationSnapshotDto, PendingResearchDeliverySnapshotDto,
    RewardOptionSnapshotDto, RosterOrderSlotSnapshotDto, RosterOrderSnapshotDto,
    RunAbnormalityResearchEntrySnapshotDto, RunAbnormalityResearchSnapshotDto,
    RunProgressionSnapshotDto, RunResourcesSnapshotDto, RunSnapshotDto, SelectedEventSnapshotDto,
    SkillCatalogCastTargetDto, SkillCatalogDeliveryKind, SkillCatalogDto, SkillCatalogSkillDto,
    SkillCatalogStepDto, SkillCatalogTileAreaDto, SkillFragmentProgressSnapshotDto,
    SnapshotErrorDto,
};
use crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee;
use crate::game::data::ItemRef;
use crate::game::resources::{ActiveNodeContent, GameState};
use crate::game::reward::RewardOption;

fn snapshot_game_error_code(error: &GameError) -> &'static str {
    match error {
        GameError::EventNotFound => "event_not_found",
        GameError::EventTypeMismatch => "event_type_mismatch",
        GameError::InvalidAction => "invalid_action",
        GameError::InvalidBattleResyncSeq { .. } => "invalid_battle_resync_seq",
        GameError::NotInShopState => "not_in_shop_state",
        GameError::NotInRewardState => "not_in_reward_state",
        GameError::ShopRerollNotAllowed => "shop_reroll_not_allowed",
        GameError::ShopItemNotFound => "shop_item_not_found",
        GameError::InventoryFull => "inventory_full",
        GameError::InventoryItemNotFound => "inventory_item_not_found",
        GameError::InventoryItemNotRemovable => "inventory_item_not_removable",
        GameError::AlreadyOwnedArtifact => "already_owned_artifact",
        GameError::InsufficientResources => "insufficient_resources",
        GameError::MissingResource(_) => "missing_resource",
        GameError::InvalidUnitStats(_) => "invalid_unit_stats",
        GameError::InvalidStaticData(_) => "invalid_static_data",
        GameError::NotImplemented(_) => "not_implemented",
        GameError::SkillFragmentIncompatible { .. } => "skill_fragment_incompatible",
        GameError::OutOfBounds => "out_of_bounds",
        GameError::PositionOccupied => "position_occupied",
        GameError::StaticObstacleBlocked => "static_obstacle_blocked",
        GameError::UnitAlreadyPlaced => "unit_already_placed",
        GameError::UnitNotFound => "unit_not_found",
    }
}

fn snapshot_game_error_message(error: &GameError) -> String {
    match error {
        GameError::EventNotFound => "event not found".to_string(),
        GameError::EventTypeMismatch => "event type mismatch".to_string(),
        GameError::InvalidAction => "invalid action".to_string(),
        GameError::InvalidBattleResyncSeq { requested, latest } => {
            format!("invalid battle resync seq: requested {requested}, latest {latest}")
        }
        GameError::NotInShopState => "not in shop state".to_string(),
        GameError::NotInRewardState => "not in reward state".to_string(),
        GameError::ShopRerollNotAllowed => "shop reroll not allowed".to_string(),
        GameError::ShopItemNotFound => "shop item not found".to_string(),
        GameError::InventoryFull => "inventory full".to_string(),
        GameError::InventoryItemNotFound => "inventory item not found".to_string(),
        GameError::InventoryItemNotRemovable => "inventory item not removable".to_string(),
        GameError::AlreadyOwnedArtifact => "already owned artifact".to_string(),
        GameError::InsufficientResources => "insufficient resources".to_string(),
        GameError::MissingResource(resource) => format!("missing resource: {resource}"),
        GameError::InvalidUnitStats(reason) => format!("invalid unit stats: {reason}"),
        GameError::InvalidStaticData(reason) => format!("invalid static data: {reason}"),
        GameError::NotImplemented(feature) => format!("not implemented: {feature}"),
        GameError::SkillFragmentIncompatible {
            fragment_id,
            failure_codes,
        } => format!(
            "skill fragment incompatible: {fragment_id}; failure code count {}",
            failure_codes.len()
        ),
        GameError::OutOfBounds => "out of bounds".to_string(),
        GameError::PositionOccupied => "position occupied".to_string(),
        GameError::StaticObstacleBlocked => "static obstacle blocked".to_string(),
        GameError::UnitAlreadyPlaced => "unit already placed".to_string(),
        GameError::UnitNotFound => "unit not found".to_string(),
    }
}

fn effective_profile_error_dto(error: &GameError) -> SnapshotErrorDto {
    SnapshotErrorDto {
        code: snapshot_game_error_code(error).to_string(),
        message: snapshot_game_error_message(error),
    }
}

fn wire_skill_fragment_effect_label(
    effect: &crate::game::data::skill_fragment_data::SkillFragmentEffectDef,
) -> &'static str {
    match effect {
        crate::game::data::skill_fragment_data::SkillFragmentEffectDef::BasicAttackModifier {
            ..
        } => "BasicAttackModifier",
        crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { .. } => {
            "ActiveSkill"
        }
    }
}

fn wire_equipment_material_type_label(
    material_type: crate::game::data::equipment_data::EquipmentMaterialType,
) -> &'static str {
    match material_type {
        crate::game::data::equipment_data::EquipmentMaterialType::Fragment => "Fragment",
        crate::game::data::equipment_data::EquipmentMaterialType::Core => "Core",
        crate::game::data::equipment_data::EquipmentMaterialType::Blueprint => "Blueprint",
        crate::game::data::equipment_data::EquipmentMaterialType::Residue => "Residue",
        crate::game::data::equipment_data::EquipmentMaterialType::Generic => "Generic",
    }
}

fn wire_employee_life_state_label(
    life_state: crate::game::employee::EmployeeLifeState,
) -> &'static str {
    match life_state {
        crate::game::employee::EmployeeLifeState::Alive => "Alive",
        crate::game::employee::EmployeeLifeState::Dead => "Dead",
    }
}

fn wire_employee_availability_label(
    availability: crate::game::employee::EmployeeAvailability,
) -> &'static str {
    match availability {
        crate::game::employee::EmployeeAvailability::Available => "Available",
        crate::game::employee::EmployeeAvailability::Unavailable => "Unavailable",
    }
}

fn wire_tier_label(tier: crate::game::enums::Tier) -> &'static str {
    match tier {
        crate::game::enums::Tier::I => "I",
        crate::game::enums::Tier::II => "II",
        crate::game::enums::Tier::III => "III",
    }
}

fn wire_trust_band_label(band: crate::game::employee_trust::TrustBand) -> &'static str {
    match band {
        crate::game::employee_trust::TrustBand::Distrust => "Distrust",
        crate::game::employee_trust::TrustBand::Uneasy => "Uneasy",
        crate::game::employee_trust::TrustBand::Neutral => "Neutral",
        crate::game::employee_trust::TrustBand::Trusting => "Trusting",
        crate::game::employee_trust::TrustBand::Devoted => "Devoted",
    }
}

fn wire_employee_trait_label(
    trait_kind: crate::game::employee_trust::EmployeeTrait,
) -> &'static str {
    match trait_kind {
        crate::game::employee_trust::EmployeeTrait::Brave => "Brave",
        crate::game::employee_trust::EmployeeTrait::Cautious => "Cautious",
        crate::game::employee_trust::EmployeeTrait::Obedient => "Obedient",
        crate::game::employee_trust::EmployeeTrait::Defiant => "Defiant",
        crate::game::employee_trust::EmployeeTrait::Comradely => "Comradely",
        crate::game::employee_trust::EmployeeTrait::SurvivalInstinct => "SurvivalInstinct",
    }
}

fn wire_trust_memory_kind_label(
    kind: crate::game::employee_trust::TrustMemoryKind,
) -> &'static str {
    match kind {
        crate::game::employee_trust::TrustMemoryKind::TreatedAfterIncapacitation => {
            "TreatedAfterIncapacitation"
        }
        crate::game::employee_trust::TrustMemoryKind::RestedBeforeDanger => "RestedBeforeDanger",
        crate::game::employee_trust::TrustMemoryKind::PromiseKept => "PromiseKept",
        crate::game::employee_trust::TrustMemoryKind::ConsistentInvestment => {
            "ConsistentInvestment"
        }
        crate::game::employee_trust::TrustMemoryKind::DeployedWhileInjured => {
            "DeployedWhileInjured"
        }
        crate::game::employee_trust::TrustMemoryKind::IncapacitatedNeglected => {
            "IncapacitatedNeglected"
        }
        crate::game::employee_trust::TrustMemoryKind::ForcedEarlyAwakening => {
            "ForcedEarlyAwakening"
        }
        crate::game::employee_trust::TrustMemoryKind::ForcedRiskFragmentUse => {
            "ForcedRiskFragmentUse"
        }
        crate::game::employee_trust::TrustMemoryKind::AllyDeathThenDanger => "AllyDeathThenDanger",
        crate::game::employee_trust::TrustMemoryKind::RestedAtSupportRest => "RestedAtSupportRest",
        crate::game::employee_trust::TrustMemoryKind::NeglectedAtSupportRest => {
            "NeglectedAtSupportRest"
        }
        crate::game::employee_trust::TrustMemoryKind::AllyDeathWitnessed => "AllyDeathWitnessed",
    }
}

fn wire_trust_event_kind_label(kind: crate::game::employee_trust::TrustEventKind) -> &'static str {
    match kind {
        crate::game::employee_trust::TrustEventKind::TreatedAfterIncapacitation => {
            "TreatedAfterIncapacitation"
        }
        crate::game::employee_trust::TrustEventKind::RestedBeforeDanger => "RestedBeforeDanger",
        crate::game::employee_trust::TrustEventKind::PromiseKept => "PromiseKept",
        crate::game::employee_trust::TrustEventKind::ConsistentInvestment => "ConsistentInvestment",
        crate::game::employee_trust::TrustEventKind::DeployedWhileInjured => "DeployedWhileInjured",
        crate::game::employee_trust::TrustEventKind::IncapacitatedNeglected => {
            "IncapacitatedNeglected"
        }
        crate::game::employee_trust::TrustEventKind::ForcedEarlyAwakening => "ForcedEarlyAwakening",
        crate::game::employee_trust::TrustEventKind::ForcedRiskFragmentUse => {
            "ForcedRiskFragmentUse"
        }
        crate::game::employee_trust::TrustEventKind::AllyDeathThenDanger => "AllyDeathThenDanger",
        crate::game::employee_trust::TrustEventKind::RestedAtSupportRest => "RestedAtSupportRest",
        crate::game::employee_trust::TrustEventKind::NeglectedAtSupportRest => {
            "NeglectedAtSupportRest"
        }
        crate::game::employee_trust::TrustEventKind::AllyDied => "AllyDied",
        crate::game::employee_trust::TrustEventKind::BeforeDangerNode => "BeforeDangerNode",
        crate::game::employee_trust::TrustEventKind::BeforeFinalNode => "BeforeFinalNode",
        crate::game::employee_trust::TrustEventKind::IncurredTrauma => "IncurredTrauma",
    }
}

impl GameCore {
    pub fn get_run_snapshot_dto(&self) -> Result<RunSnapshotDto, GameError> {
        let map = self
            .state
            .run
            .as_ref()
            .map(|run| run.map_progression.view(&run.map));
        let run_progression = self.state.run.as_ref().map(|state| {
            let run = &state.run_progression;
            RunProgressionSnapshotDto {
                run_seed: run.run_seed,
                game_mode: run.game_mode,
                floor_index: run.floor_index(),
                max_floors: run.max_floors(),
                current_floor_seed: run.current_floor_seed(),
            }
        });
        let abnormality_research = self.state.run.as_ref().map(|state| {
            let mut entries = state
                .abnormality_research
                .entries
                .iter()
                .map(|(abnormality_id, entry)| {
                    let response_complete_skill_fragment_id = self
                        .game_data
                        .abnormality_data
                        .get_by_id(abnormality_id)
                        .and_then(|abnormality| {
                            abnormality.response_complete_skill_fragment_id.clone()
                        });
                    RunAbnormalityResearchEntrySnapshotDto {
                        abnormality_id: abnormality_id.clone(),
                        research_points: entry.research_points,
                        research_required: entry.research_required,
                        response_complete: entry.response_complete,
                        suppression_wins: entry.suppression_wins,
                        last_encountered_floor: entry.last_encountered_floor,
                        completed_at_floor: entry.completed_at_floor,
                        unique_fragment_granted: entry.unique_fragment_granted,
                        response_complete_skill_fragment_id,
                    }
                })
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| left.abnormality_id.cmp(&right.abnormality_id));
            RunAbnormalityResearchSnapshotDto {
                entries,
                all_response_complete: state.abnormality_research.all_response_complete(),
            }
        });
        let map_navigation = self.state.run.as_ref().and_then(|state| {
            let progression = &state.map_progression;
            progression
                .current_node_id
                .map(|current_node_id| MapNavigationSnapshotDto {
                    current_node_id,
                    selectable_node_ids: progression.selectable_node_ids(&state.map),
                })
        });
        let enkephalin = self.state.enkephalin.amount;
        Ok(RunSnapshotDto {
            game_state: self.game_state_name().to_string(),
            game_state_context: self.game_state_context_dto(),
            allowed_actions: self.get_allowed_actions(),
            run_checkpoint: self.state.run_checkpoint.to_dto(),
            run_progression,
            abnormality_research,
            map_navigation,
            map,
            current_node_session: self.state.node_session.clone(),
            selected_event: self.get_selected_event_snapshot_dto()?,
            skill_catalog: self.get_skill_catalog_snapshot_dto(),
            roster: self.get_employee_roster_snapshot_dto()?,
            roster_order: self.get_roster_order_snapshot_dto()?,
            inventory: self.get_inventory_snapshot_dto()?,
            resources: RunResourcesSnapshotDto { enkephalin },
        })
    }

    pub fn get_run_snapshot_json(&self) -> Result<Value, GameError> {
        serde_json::to_value(self.get_run_snapshot_dto()?)
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))
    }

    fn game_state_context_dto(&self) -> GameStateContextDto {
        match self.get_state() {
            GameState::NotStarted => GameStateContextDto::NotStarted,
            GameState::SelectingStarterEmployees => {
                GameStateContextDto::SelectingStarterEmployees {
                    required_count: self.run_policy().setup.starter_employee_count,
                    candidates: self.state.starter_candidates.clone(),
                }
            }
            GameState::ViewingMap => GameStateContextDto::ViewingMap,
            GameState::NodeConfirm {
                node_id,
                kind_id,
                category,
            } => {
                let run = self.state.run.as_ref();
                let combat_preview = run.and_then(|run| run.combat_previews.get(&node_id).cloned());
                let abnormality_attempt = run
                    .filter(|_| {
                        matches!(
                            category,
                            crate::game::map::MapNodeCategory::Combat
                                | crate::game::map::MapNodeCategory::Boss
                        )
                    })
                    .map(|run| run.abnormality_attempt_dto(node_id));
                GameStateContextDto::NodeConfirm {
                    node_id,
                    kind_id,
                    category,
                    combat_preview,
                    abnormality_attempt,
                    omen: run.and_then(|run| run.boss_omen.node_confirm_omen(node_id)),
                }
            }
            GameState::InNode {
                node_id,
                kind_id,
                category,
            } => GameStateContextDto::InNode {
                node_id,
                kind_id,
                category,
            },
            GameState::InShop { shop_uuid } => GameStateContextDto::InShop { shop_uuid },
            GameState::InReward { reward_uuid } => GameStateContextDto::InReward { reward_uuid },
            GameState::InRewardClaimed { reward_uuid } => {
                GameStateContextDto::InRewardClaimed { reward_uuid }
            }
            GameState::CombatResult { battle_uuid } => {
                GameStateContextDto::CombatResult { battle_uuid }
            }
            GameState::InBattle { battle_uuid } => {
                let active = self.state.active_battle.as_ref();
                let abnormality_attempt = active.and_then(|battle| {
                    self.state.run.as_ref().map(|run| {
                        run.abnormality_attempt_dto(crate::game::map::MapNodeId(battle.battle_uuid))
                    })
                });
                let can_retreat = active.is_some_and(|battle| !battle.execution.is_finished());
                GameStateContextDto::InBattle {
                    battle_uuid,
                    node_type: active.map(|battle| battle.node_type),
                    mission_variant: active.map(|battle| battle.mission_variant),
                    encounter_id: active.map(|battle| battle.encounter_id.clone()),
                    combat_preview: active.map(|battle| battle.combat_preview.clone()),
                    last_pushed_event_log_seq: active
                        .and_then(|battle| battle.last_pushed_event_log_seq),
                    deployment: active
                        .and_then(|battle| battle.live_deployment_dto(&self.state.roster)),
                    playback: active.map(|battle| battle.playback_state()),
                    abnormality_attempt,
                    can_retreat,
                }
            }
            GameState::GameOver => GameStateContextDto::GameOver,
            GameState::RunComplete => GameStateContextDto::RunComplete,
            GameState::RunFailed { reason } => GameStateContextDto::RunFailed { reason },
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
                    |(target, range_policy, defense_tile_range, air_capable)| {
                        SkillCatalogCastTargetDto {
                            target_policy: target.clone(),
                            range_policy,
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
                                target_policy: step.target.clone(),
                                range_policy: step.range_policy,
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
            range_source_of_truth: "range_previews.final_cells".to_string(),
            skills,
        }
    }

    fn skill_fragment_progress_snapshot(
        progress: &crate::game::skill_fragment::SkillFragmentProgress,
    ) -> SkillFragmentProgressSnapshotDto {
        SkillFragmentProgressSnapshotDto {
            research_progress: progress.research_progress,
            research_completion_count: progress.research_completion_count,
            upgrade_level: progress.upgrade_level,
            awakening_progress: progress.awakening_progress,
            awakening_available: progress.awakening_available,
            awakened: progress.awakened,
        }
    }

    pub fn get_inventory_snapshot_dto(&self) -> Result<InventorySnapshotDto, GameError> {
        let inventory = self.inventory()?;
        let skill_fragments = self
            .state
            .skill_fragments
            .owned_ids()
            .map(|fragment_id| {
                let metadata = self.game_data.skill_fragment_data.get_by_id(fragment_id);
                let progress = self.state.skill_fragments.progress(fragment_id);
                InventorySkillFragmentSnapshotDto {
                    id: fragment_id.clone(),
                    count: self.state.skill_fragments.count(fragment_id),
                    name: metadata.map(|fragment| fragment.name.clone()),
                    effect: metadata
                        .map(|fragment| wire_skill_fragment_effect_label(&fragment.effect).into()),
                    requirements: metadata.map(|fragment| fragment.compatibility.clone()),
                    progress: Self::skill_fragment_progress_snapshot(&progress),
                }
            })
            .collect::<Vec<_>>();
        let skill_fragment_progress = self
            .state
            .skill_fragments
            .progress_entries()
            .map(|(fragment_id, progress)| {
                let metadata = self.game_data.skill_fragment_data.get_by_id(fragment_id);
                InventorySkillFragmentProgressSnapshotDto {
                    id: fragment_id.clone(),
                    owned_count: self.state.skill_fragments.count(fragment_id),
                    name: metadata.map(|fragment| fragment.name.clone()),
                    progress: Self::skill_fragment_progress_snapshot(&progress),
                }
            })
            .collect::<Vec<_>>();

        let mut equipments = inventory
            .equipments
            .iter()
            .map(|owned| InventoryEquipmentSnapshotDto {
                instance_uuid: owned.instance_uuid,
                item: crate::game::resources::EquipmentItemDto::from_owned_equipment(owned),
                equipped_to: owned.equipped_to,
            })
            .collect::<Vec<_>>();
        equipments.sort_by(|left, right| left.instance_uuid.cmp(&right.instance_uuid));

        let mut artifacts = inventory
            .artifacts
            .iter()
            .map(|artifact| {
                crate::game::resources::ArtifactItemDto::from_metadata(artifact.as_ref())
            })
            .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| left.uuid.cmp(&right.uuid));

        let mut consumables = inventory
            .consumables
            .iter()
            .map(crate::game::resources::ConsumableItemDto::from_owned_consumable)
            .collect::<Vec<_>>();
        consumables.sort_by(|left, right| left.uuid.cmp(&right.uuid));

        let mut equipment_materials = inventory
            .equipment_materials
            .iter()
            .map(|(material_id, amount)| {
                let metadata = self
                    .game_data
                    .equipment_data
                    .get_material_by_id(material_id);
                InventoryEquipmentMaterialSnapshotDto {
                    material_id: material_id.clone(),
                    amount: *amount,
                    name: metadata.map(|material| material.name.clone()),
                    description: metadata.map(|material| material.description.clone()),
                    material_type: metadata.map(|material| {
                        wire_equipment_material_type_label(material.material_type).into()
                    }),
                    rarity: metadata.map(|material| material.rarity),
                    equipment_type: metadata.and_then(|material| material.equipment_type),
                }
            })
            .collect::<Vec<_>>();
        equipment_materials.sort_by(|left, right| left.material_id.cmp(&right.material_id));

        Ok(InventorySnapshotDto {
            equipments,
            equipment_materials,
            artifacts,
            consumables,
            skill_fragments,
            skill_fragment_progress,
            pending_research_deliveries: self
                .state
                .skill_fragments
                .pending_research_deliveries()
                .map(|(fragment_id, count)| PendingResearchDeliverySnapshotDto {
                    fragment_id: fragment_id.clone(),
                    count,
                })
                .collect::<Vec<_>>(),
            fragment_dust: self.state.skill_fragments.fragment_dust(),
        })
    }

    pub fn get_inventory_snapshot_json(&self) -> Result<Value, GameError> {
        serde_json::to_value(self.get_inventory_snapshot_dto()?)
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))
    }

    pub fn get_roster_order_snapshot_dto(&self) -> Result<RosterOrderSnapshotDto, GameError> {
        let roster_order = self.roster_order()?;

        let slots = roster_order
            .slots
            .iter()
            .enumerate()
            .map(|(slot, unit_uuid)| RosterOrderSlotSnapshotDto {
                slot,
                unit_uuid: *unit_uuid,
            })
            .collect::<Vec<_>>();

        Ok(RosterOrderSnapshotDto {
            max_slots: roster_order.max_slots,
            slots,
        })
    }

    pub fn get_roster_order_snapshot_json(&self) -> Result<Value, GameError> {
        serde_json::to_value(self.get_roster_order_snapshot_dto()?)
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))
    }

    pub fn get_employee_roster_snapshot_dto(&self) -> Result<EmployeeRosterSnapshotDto, GameError> {
        let roster = self.roster()?;
        let inventory = self.inventory()?;
        let roster_order = self.roster_order().ok();

        let mut employees = roster
            .iter()
            .map(|employee| -> Result<EmployeeSnapshotDto, GameError> {
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
                    Ok(profile) => (Some(profile), None),
                    Err(error) => (None, Some(effective_profile_error_dto(&error))),
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
                        Ok(EmployeeSkillFragmentCompatibilitySnapshotDto {
                            id: fragment_id.clone(),
                            name: metadata.map(|fragment| fragment.name.clone()),
                            is_compatible: report.is_compatible,
                            failure_codes: report.failure_codes,
                            requirements: metadata.map(|fragment| fragment.compatibility.clone()),
                        })
                    })
                    .collect::<Result<Vec<_>, GameError>>()?;
                Ok(EmployeeSnapshotDto {
                    uuid: employee.uuid,
                    name: employee.name.clone(),
                    level: employee.level,
                    experience: employee.experience,
                    life_state: wire_employee_life_state_label(employee.life_state).into(),
                    availability: wire_employee_availability_label(employee.availability).into(),
                    available_for_combat: employee.is_available_for_combat(),
                    trauma: employee.trauma,
                    health: EmployeeHealthSnapshotDto {
                        current_hp: employee.health.current_hp,
                        max_hp: employee.health.max_hp,
                    },
                    injuries: employee
                        .injuries
                        .iter()
                        .map(|injury| EmployeeInjurySnapshotDto {
                            id: injury.id.clone(),
                            severity: injury.severity,
                        })
                        .collect::<Vec<_>>(),
                    trust: EmployeeTrustSnapshotDto {
                        score: employee.trust.score,
                        band: wire_trust_band_label(employee.trust.band()).into(),
                        traits: employee
                            .trust
                            .traits
                            .iter()
                            .map(|trait_kind| wire_employee_trait_label(*trait_kind).into())
                            .collect::<Vec<_>>(),
                        memories: employee
                            .trust
                            .memories
                            .iter()
                            .map(|memory| EmployeeTrustMemorySnapshotDto {
                                kind: wire_trust_memory_kind_label(memory.kind).into(),
                                intensity: memory.intensity,
                                remaining_nodes: memory.remaining_nodes,
                            })
                            .collect::<Vec<_>>(),
                        recent_reactions: employee
                            .trust
                            .recent_reactions
                            .iter()
                            .map(|reaction| EmployeeTrustReactionSnapshotDto {
                                event: wire_trust_event_kind_label(reaction.event).into(),
                                trust_delta: reaction.trust_delta,
                                cue_count: reaction.cue_count,
                                combat_modifier_count: reaction.combat_modifier_count,
                                trauma_modifier_count: reaction.trauma_modifier_count,
                            })
                            .collect::<Vec<_>>(),
                    },
                    combat_profile: EmployeeCombatProfileSnapshotDto {
                        battle_tier: wire_tier_label(
                            employee.battle_tier(self.game_data.run_policy.as_ref()),
                        )
                        .into(),
                        base_stats: employee.combat_profile.battle_profile.stats,
                        basic_attack: employee.combat_profile.battle_profile.basic_attack.clone(),
                        deployment_affinity: employee
                            .combat_profile
                            .battle_profile
                            .deployment_affinity,
                        effective_stats: effective_profile.as_ref().map(|profile| profile.stats),
                        effective_basic_attack: effective_profile
                            .as_ref()
                            .map(|profile| profile.basic_attack.clone()),
                        effective_weapon_profile: effective_profile
                            .as_ref()
                            .and_then(|profile| profile.weapon_profile.clone()),
                        effective_skill_id: effective_profile
                            .as_ref()
                            .and_then(|profile| profile.skill_id.clone()),
                        effective_deployment_affinity: effective_profile
                            .as_ref()
                            .map(|profile| profile.deployment_affinity),
                        effective_profile_error,
                    },
                    skill_fragments: EmployeeSkillFragmentsSnapshotDto {
                        equipped: employee
                            .skill_fragments
                            .equipped_ids()
                            .iter()
                            .map(|fragment_id| {
                                let metadata =
                                    self.game_data.skill_fragment_data.get_by_id(fragment_id);
                                EmployeeSkillFragmentBriefSnapshotDto {
                                    id: fragment_id.clone(),
                                    name: metadata.map(|fragment| fragment.name.clone()),
                                    effect: metadata.map(|fragment| {
                                        wire_skill_fragment_effect_label(&fragment.effect).into()
                                    }),
                                }
                            })
                            .collect::<Vec<_>>(),
                        baseline_ids: employee.skill_fragments.baseline_ids().to_vec(),
                        active_fragment_id: employee.skill_fragments.active_fragment_id().cloned(),
                        equipped_ids: employee.skill_fragments.equipped_ids(),
                        compatibility: skill_fragment_compatibility,
                    },
                    active_consumable_modifier: employee.active_consumable_modifier.as_ref().map(
                        |modifier| ActiveConsumableModifierSnapshotDto {
                            source_item_uuid: modifier.source_item_uuid,
                            definition_id: modifier.definition_id.clone(),
                            name: modifier.name.clone(),
                            tier: modifier.tier,
                            duration_policy: modifier.duration_policy,
                            remaining_combat_nodes: modifier.remaining_combat_nodes,
                            effect: modifier.effect.clone(),
                        },
                    ),
                    equipped_items: employee
                        .loadout
                        .item_slot
                        .iter()
                        .map(|equipped| {
                            let equipment = inventory.equipments.get_item(&equipped.instance_uuid);
                            EmployeeEquippedItemSnapshotDto {
                                instance_uuid: equipped.instance_uuid,
                                base_uuid: equipped.base_uuid,
                                equipment_type: equipped.equipment_type,
                                definition_id: equipment.map(|item| item.meta.id.clone()),
                                name: equipment.map(|item| item.meta.name.clone()),
                                weapon_profile: equipment
                                    .and_then(|item| item.meta.weapon_profile.clone()),
                                bound: equipment.map(|item| item.meta.bound).unwrap_or(false),
                                can_unequip: equipment
                                    .map(|item| !item.meta.bound)
                                    .unwrap_or(false),
                                cannot_unequip_reason: equipment.and_then(|item| {
                                    item.meta
                                        .bound
                                        .then(|| item.meta.cannot_unequip_reason.clone())
                                }),
                            }
                        })
                        .collect::<Vec<_>>(),
                    roster_slot,
                })
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        employees.sort_by(|left, right| left.uuid.cmp(&right.uuid));

        Ok(EmployeeRosterSnapshotDto {
            employees,
            available_employee_ids: roster.available_employee_ids(),
        })
    }

    pub fn get_employee_roster_snapshot_json(&self) -> Result<Value, GameError> {
        serde_json::to_value(self.get_employee_roster_snapshot_dto()?)
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))
    }

    fn display_items_snapshot_dto(
        &self,
        item_uuids: &[Uuid],
    ) -> Result<Vec<DisplayItemSnapshotDto>, GameError> {
        item_uuids
            .iter()
            .copied()
            .map(|item_uuid| self.display_item_snapshot_dto(item_uuid))
            .collect()
    }

    fn display_item_snapshot_dto(
        &self,
        item_uuid: Uuid,
    ) -> Result<DisplayItemSnapshotDto, GameError> {
        let item = self.game_data.item(&item_uuid).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "selected event references missing item uuid {item_uuid}"
            ))
        })?;

        let value = match item {
            ItemRef::Equipment(meta) => {
                let equipment_type = equipment_type_display_key(meta.equipment_type);
                DisplayItemSnapshotDto::Equipment(DisplayEquipmentItemSnapshotDto {
                    uuid: item_uuid,
                    kind: "equipment".to_string(),
                    definition_id: meta.id.clone(),
                    id: meta.id.clone(),
                    name: meta.name.clone(),
                    rarity: meta.rarity,
                    price: meta.price,
                    item_type: equipment_type.to_string(),
                    equipment_type: equipment_type.to_string(),
                    description: String::new(),
                })
            }
            ItemRef::Artifact(meta) => {
                DisplayItemSnapshotDto::Artifact(DisplayArtifactItemSnapshotDto {
                    uuid: item_uuid,
                    kind: "artifact".to_string(),
                    definition_id: meta.id.clone(),
                    id: meta.id.clone(),
                    name: meta.name.clone(),
                    rarity: meta.rarity,
                    price: meta.price,
                    item_type: "artifact".to_string(),
                    artifact_type: "artifact".to_string(),
                    effect_id: meta.id.clone(),
                    description: meta.description.clone(),
                })
            }
            ItemRef::Consumable(meta) => {
                DisplayItemSnapshotDto::Consumable(DisplayConsumableItemSnapshotDto {
                    uuid: item_uuid,
                    definition_id: meta.id.clone(),
                    name: meta.name.clone(),
                    description: meta.description.clone(),
                    tier: meta.tier,
                    rarity: meta.rarity,
                    price: meta.price,
                    target_policy: meta.target_policy,
                    duration_policy: meta.duration_policy,
                    effect: meta.effect.clone(),
                    kind: "consumable".to_string(),
                    id: meta.id.clone(),
                    item_type: "consumable".to_string(),
                })
            }
        };

        Ok(value)
    }

    pub fn get_selected_event_snapshot_dto(
        &self,
    ) -> Result<Option<SelectedEventSnapshotDto>, GameError> {
        let Some(selected) = self.state.active_node_content.as_ref() else {
            return Ok(None);
        };

        let value = match selected {
            ActiveNodeContent::Shop(shop) => SelectedEventSnapshotDto::Shop {
                id: shop.id.clone(),
                name: shop.name.clone(),
                uuid: shop.uuid,
                shop_type: shop.shop_type,
                can_reroll: shop.can_reroll,
                visible_items: self.display_items_snapshot_dto(&shop.visible_items)?,
                visible_item_uuids: shop.visible_items.clone(),
            },
            ActiveNodeContent::Reward(reward) => SelectedEventSnapshotDto::Reward {
                stage_uuid: reward.stage_uuid,
                mode: reward.mode,
                rewards: reward
                    .rewards
                    .iter()
                    .map(display_reward_option_snapshot_dto)
                    .collect::<Vec<_>>(),
                selected_reward_uuid: reward.selected_reward_uuid,
                can_skip: reward.can_skip,
            },
            ActiveNodeContent::Event(event) => SelectedEventSnapshotDto::Event {
                node_id: event.node_id,
                event: self.event_scene_snapshot(event)?,
            },
            ActiveNodeContent::Support(support) => SelectedEventSnapshotDto::Support {
                node_id: support.node_id,
                support_mode: support.support_mode,
                support_type: support.support_type,
                choices: support.choices.clone(),
                selected_support_type: support.selected_support_type,
            },
            ActiveNodeContent::Maintenance(maintenance) => SelectedEventSnapshotDto::Maintenance {
                node_id: maintenance.node_id,
                maintenance_options: self.maintenance_options(),
            },
            ActiveNodeContent::HeadquartersContact(headquarters) => {
                SelectedEventSnapshotDto::HeadquartersContact {
                    node_id: headquarters.node_id,
                    options: headquarters.options.clone(),
                    recruitment_candidates: headquarters.recruitment_candidates.clone(),
                    shop_pool_id: headquarters.shop_pool_id.clone(),
                }
            }
            ActiveNodeContent::CombatBattle(battle) => SelectedEventSnapshotDto::CombatBattle {
                primary_abnormality_id: battle.primary_abnormality_id.clone(),
                encounter_id: battle.encounter_id.clone(),
                node_type: battle.node_type,
                mission_variant: battle.mission_variant,
                abnormality_uuid: battle.abnormality_uuid,
                winner: battle.winner,
                reward_mode: battle.reward_mode,
                rewards: battle
                    .rewards
                    .iter()
                    .map(display_reward_option_snapshot_dto)
                    .collect::<Vec<_>>(),
                result_stats: battle.result_stats.clone(),
                bonus_objectives: battle.bonus_objectives.clone(),
                has_event_log: true,
            },
        };

        Ok(Some(value))
    }

    pub fn get_selected_event_snapshot_json(&self) -> Result<Option<Value>, GameError> {
        self.get_selected_event_snapshot_dto()?
            .map(|selected| {
                serde_json::to_value(selected).map_err(|error| {
                    GameError::InvalidStaticData(format!(
                        "failed to serialize selected event snapshot: {error}"
                    ))
                })
            })
            .transpose()
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

fn display_reward_option_snapshot_dto(reward: &RewardOption) -> RewardOptionSnapshotDto {
    RewardOptionSnapshotDto {
        uuid: reward.uuid,
        kind: "reward".to_string(),
        id: reward.id.clone(),
        name: reward.name.clone(),
        rarity: None,
        price: 0,
        description: reward.description.clone(),
        icon: reward.icon.clone(),
        grant_kinds: reward.grant_kinds(),
        effects: reward.effects.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_snapshot_string_labels_are_explicit_contract_values() {
        assert_eq!(
            wire_skill_fragment_effect_label(
                &crate::game::data::skill_fragment_data::SkillFragmentEffectDef::BasicAttackModifier {
                    attack_bonus: 10,
                    attack_interval_ms_reduction: 50,
                },
            ),
            "BasicAttackModifier"
        );
        assert_eq!(
            wire_equipment_material_type_label(
                crate::game::data::equipment_data::EquipmentMaterialType::Fragment
            ),
            "Fragment"
        );
        assert_eq!(
            wire_employee_life_state_label(crate::game::employee::EmployeeLifeState::Alive),
            "Alive"
        );
        assert_eq!(
            wire_employee_availability_label(
                crate::game::employee::EmployeeAvailability::Available
            ),
            "Available"
        );
        assert_eq!(wire_tier_label(crate::game::enums::Tier::III), "III");
        assert_eq!(
            wire_trust_band_label(crate::game::employee_trust::TrustBand::Devoted),
            "Devoted"
        );
        assert_eq!(
            wire_employee_trait_label(crate::game::employee_trust::EmployeeTrait::SurvivalInstinct),
            "SurvivalInstinct"
        );
        assert_eq!(
            wire_trust_memory_kind_label(
                crate::game::employee_trust::TrustMemoryKind::AllyDeathWitnessed
            ),
            "AllyDeathWitnessed"
        );
        assert_eq!(
            wire_trust_event_kind_label(
                crate::game::employee_trust::TrustEventKind::IncurredTrauma
            ),
            "IncurredTrauma"
        );
    }
}
