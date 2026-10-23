use serde_json::Value;
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{
    ActiveConsumableModifierSnapshotDto, CombatRecoveryRestrictionSnapshotDto,
    DisplayConsumableItemSnapshotDto, DisplayEquipmentItemSnapshotDto, DisplayItemSnapshotDto,
    EmployeeCombatProfileSnapshotDto, EmployeeConditionSnapshotDto,
    EmployeeEligibilitySnapshotDto, EmployeeEquippedItemSnapshotDto,
    EmployeeRosterSnapshotDto,
    EmployeeSkillFragmentBriefSnapshotDto, EmployeeSkillFragmentCompatibilitySnapshotDto,
    EmployeeSkillFragmentsSnapshotDto, EmployeeSnapshotDto,
    EncounterCompendiumAbnormalitySnapshotDto, EncounterCompendiumCorrosionSnapshotDto,
    EncounterCompendiumSnapshotDto, GameError, GameStateContextDto,
    InventoryEquipmentMaterialSnapshotDto, InventoryEquipmentSnapshotDto,
    InventorySkillFragmentSnapshotDto, InventorySnapshotDto, MapNavigationSnapshotDto,
    PlayerProfileSnapshotDto, RewardOptionSnapshotDto, RunProgressionSnapshotDto,
    RunResourcesSnapshotDto, RunSnapshotDto, SelectedEventSnapshotDto, SkillCatalogCastTargetDto,
    SkillCatalogDeliveryKind, SkillCatalogDto, SkillCatalogSkillDto, SkillCatalogStepDto,
    SkillCatalogTileAreaDto, SnapshotErrorDto, VisualNovelChoicePreviewDto,
    VisualNovelChoiceSnapshotDto, VisualNovelPresentationModeDto, VisualNovelSnapshotDto,
    VisualNovelSourceDto, VisualNovelSpeakerDto, VisualNovelSpeakerPositionDto,
};
use crate::game::combat_setup::player_spawns::effective_combat_profile_for_employee_with_relics;
use crate::game::data::visual_novel_data::{
    VisualNovelBackground, VisualNovelChoicePreview, VisualNovelDefinition,
    VisualNovelSceneDefinition, VisualNovelSceneNext, VisualNovelSpeakerPosition,
};
use crate::game::data::ItemRef;
use crate::game::resources::{ActiveNodeContent, GameState};
use crate::game::reward::RewardOption;
use crate::game::visual_novel::{VisualNovelSessionPresentationMode, VisualNovelSource};

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
const COMBAT_TRACE_CAPABILITY_AVAILABLE: bool = true;
#[cfg(not(any(test, debug_assertions, feature = "test-fixtures")))]
const COMBAT_TRACE_CAPABILITY_AVAILABLE: bool = false;

fn snapshot_game_error_message(error: &GameError) -> String {
    match error {
        GameError::BattleRuntimeFault(fault) => {
            format!("{} at {}", fault.code.as_str(), fault.site)
        }
        GameError::ValueExpressionFault(fault) => fault.to_string(),
        GameError::EventNotFound => "event not found".to_string(),
        GameError::EventTypeMismatch => "event type mismatch".to_string(),
        GameError::InvalidAction => "invalid action".to_string(),
        GameError::TutorialCompletionRequired { game_mode } => {
            format!("tutorial completion required for {game_mode:?}")
        }
        GameError::ActionUnavailable { action, reason } => {
            format!("action unavailable: {action:?} ({reason:?})")
        }
        GameError::WeaponRequired => "weapon required".to_string(),
        GameError::InvalidBattleResyncSeq { requested, latest } => {
            format!("invalid battle resync seq: requested {requested}, latest {latest}")
        }
        GameError::BattleSnapshotResyncRequired {
            requested,
            oldest_available_after_seq,
            latest,
        } => format!(
            "battle snapshot resync required: requested {requested}, oldest available after seq \
             {oldest_available_after_seq}, latest {latest}"
        ),
        GameError::BattleStateRevisionOverflow => "battle state revision overflow".to_string(),
        GameError::MissingBattleStateAuthority { current } => format!(
            "missing battle state authority: current {}@{}",
            current.battle_uuid, current.battle_state_revision
        ),
        GameError::StaleBattleStateAuthority { expected, current } => format!(
            "stale battle state authority: expected {}@{}, current {}@{}",
            expected.battle_uuid,
            expected.battle_state_revision,
            current.battle_uuid,
            current.battle_state_revision
        ),
        GameError::NotInShopState => "not in shop state".to_string(),
        GameError::NotInRewardState => "not in reward state".to_string(),
        GameError::ShopItemNotFound => "shop item not found".to_string(),
        GameError::InventoryFull => "inventory full".to_string(),
        GameError::InventoryItemNotFound => "inventory item not found".to_string(),
        GameError::InventoryItemNotRemovable => "inventory item not removable".to_string(),
        GameError::InsufficientResources => "insufficient resources".to_string(),
        GameError::MissingResource(resource) => format!("missing resource: {resource}"),
        GameError::InvalidUnitStats(reason) => format!("invalid unit stats: {reason}"),
        GameError::InvalidStaticData(reason) | GameError::InvalidCommandData(reason) => {
            format!("invalid static data: {reason}")
        }
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
        code: error.code().to_string(),
        message: snapshot_game_error_message(error),
    }
}

fn visual_novel_scene<'a>(
    visual_novel: &'a VisualNovelDefinition,
    scene_id: &str,
) -> Result<&'a VisualNovelSceneDefinition, GameError> {
    visual_novel.scene_by_id(scene_id).ok_or_else(|| {
        GameError::InvalidStaticData(format!(
            "visual novel '{}' missing scene '{}'",
            visual_novel.id.as_str(),
            scene_id
        ))
    })
}

fn visual_novel_background_id(background: &VisualNovelBackground) -> Option<String> {
    match background {
        VisualNovelBackground::CurrentScreenDimmed => None,
        VisualNovelBackground::Resource { background_id } => Some(background_id.clone()),
    }
}

fn visual_novel_choice_preview_dto(
    preview: &VisualNovelChoicePreview,
) -> VisualNovelChoicePreviewDto {
    VisualNovelChoicePreviewDto {
        risk: preview.risk.clone(),
        starts_combat: preview.starts_combat,
        reward_hint_id: preview.reward_hint_id.clone(),
        penalty_hint_id: preview.penalty_hint_id.clone(),
    }
}

fn visual_novel_source_dto(source: &VisualNovelSource) -> VisualNovelSourceDto {
    match source {
        VisualNovelSource::EventNode { node_id } => {
            VisualNovelSourceDto::EventNode { node_id: *node_id }
        }
        VisualNovelSource::HqBriefing => VisualNovelSourceDto::HqBriefing,
        VisualNovelSource::Map => VisualNovelSourceDto::Map,
        VisualNovelSource::Loadout => VisualNovelSourceDto::Loadout,
        VisualNovelSource::Battle { battle_uuid } => VisualNovelSourceDto::Battle {
            battle_uuid: *battle_uuid,
        },
        VisualNovelSource::Tutorial => VisualNovelSourceDto::Tutorial,
        VisualNovelSource::Radio => VisualNovelSourceDto::Radio,
        VisualNovelSource::Cutscene => VisualNovelSourceDto::Cutscene,
    }
}

fn visual_novel_speaker_position_dto(
    position: VisualNovelSpeakerPosition,
) -> VisualNovelSpeakerPositionDto {
    match position {
        VisualNovelSpeakerPosition::FarLeft => VisualNovelSpeakerPositionDto::FarLeft,
        VisualNovelSpeakerPosition::Left => VisualNovelSpeakerPositionDto::Left,
        VisualNovelSpeakerPosition::Center => VisualNovelSpeakerPositionDto::Center,
        VisualNovelSpeakerPosition::Right => VisualNovelSpeakerPositionDto::Right,
        VisualNovelSpeakerPosition::FarRight => VisualNovelSpeakerPositionDto::FarRight,
    }
}

fn visual_novel_session_presentation_mode_dto(
    mode: VisualNovelSessionPresentationMode,
) -> VisualNovelPresentationModeDto {
    match mode {
        VisualNovelSessionPresentationMode::Cinematic => VisualNovelPresentationModeDto::Cinematic,
        VisualNovelSessionPresentationMode::Radio => VisualNovelPresentationModeDto::Radio,
        VisualNovelSessionPresentationMode::Compact => VisualNovelPresentationModeDto::Compact,
        VisualNovelSessionPresentationMode::NonBlocking => {
            VisualNovelPresentationModeDto::NonBlocking
        }
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
        crate::game::data::equipment_data::EquipmentMaterialType::EquipmentDust => "EquipmentDust",
        crate::game::data::equipment_data::EquipmentMaterialType::AbnormalityPart => "AbnormalityPart",
        crate::game::data::equipment_data::EquipmentMaterialType::RevivalCatalyst => "RevivalCatalyst",
        crate::game::data::equipment_data::EquipmentMaterialType::Fragment => "Fragment",
        crate::game::data::equipment_data::EquipmentMaterialType::Core => "Core",
        crate::game::data::equipment_data::EquipmentMaterialType::Blueprint => "Blueprint",
        crate::game::data::equipment_data::EquipmentMaterialType::Residue => "Residue",
        crate::game::data::equipment_data::EquipmentMaterialType::Generic => "Generic",
    }
}

fn wire_tier_label(tier: crate::game::enums::Tier) -> &'static str {
    match tier {
        crate::game::enums::Tier::I => "I",
        crate::game::enums::Tier::II => "II",
        crate::game::enums::Tier::III => "III",
    }
}

impl GameCore {
    pub fn get_run_snapshot_dto(&self) -> Result<RunSnapshotDto, GameError> {
        let map = self.current_map_view().ok();
        let run_progression = self.state.run.as_ref().map(|state| {
            let run = &state.run_progression;
            RunProgressionSnapshotDto {
                run_seed: run.run_seed,
                instance_namespace: run.instance_namespace,
                expedition_instance: run.expedition_instance,
                deterrence: run.deterrence,
                pending_continuation: run.pending_continuation,
                game_mode: run.game_mode,
                floor_index: run.floor_index(),
                max_floors: run.max_floors(),
                current_floor_seed: run.current_floor_seed(),
            }
        });
        let map_navigation = self.state.run.as_ref().and_then(|state| {
            let progression = &state.map_progression;
            progression
                .current_node_id
                .map(|current_node_id| MapNavigationSnapshotDto {
                    current_node_id,
                    selectable_node_ids: self.selectable_map_node_ids(),
                })
        });
        let monolith = None;
        let enkephalin = self.state.enkephalin.amount;
        let encounter_compendium = self.get_encounter_compendium_snapshot_dto()?;
        let player_profile = PlayerProfileSnapshotDto {
            lifecycle: crate::authoring::clone_shape::controlled_clone(&self.state.player_profile.lifecycle)?,
            highest_floor_reached: self.state.player_profile.highest_floor_reached,
            longest_deathless_floor_streak: self
                .state
                .player_profile
                .longest_deathless_floor_streak,
            all_live_final_bosses_defeated: self
                .state
                .player_profile
                .all_live_final_bosses_defeated,
        };
        let combat_recovery_restriction = self.state.run.as_ref().and_then(|run| {
            run.combat_recovery_restriction.as_ref().map(|restriction| {
                CombatRecoveryRestrictionSnapshotDto {
                    reason: crate::game::resources::RunFailureReason::NoDeployableEmployees,
                    allowed_node_ids: restriction.allowed_node_ids.clone(),
                }
            })
        });
        Ok(RunSnapshotDto {
            game_state: self.game_state_name().to_string(),
            game_state_context: self.game_state_context_dto(self.stress_combat_view()?),
            action_capabilities: self.get_action_capabilities(),
            run_checkpoint: self
                .state
                .run_checkpoint
                .to_dto(self.checkpoint_payload_is_loadable()),
            combat_recovery_restriction,
            run_progression,
            monolith,
            revival: self.revival_snapshot(),
            player_profile,
            encounter_compendium,
            map_navigation,
            map,
            facility: self.facility_view()?,
            current_node_session: self.state.node_session.clone().map(|s| {
                if matches!(self.state.lifecycle.phase(), GameState::NodeConfirm { .. }) { self.facility_session_view(s) } else { s }
            }),
            visual_novel: self.visual_novel_snapshot_dto()?,
            selected_event: self.get_selected_event_snapshot_dto()?,
            skill_catalog: self.get_skill_catalog_snapshot_dto(),
            roster: self.get_employee_roster_snapshot_dto()?,
            inventory: self.get_inventory_snapshot_dto()?,
            resources: RunResourcesSnapshotDto { enkephalin },
        })
    }

    fn get_encounter_compendium_snapshot_dto(
        &self,
    ) -> Result<EncounterCompendiumSnapshotDto, GameError> {
        self.state
            .player_profile
            .encounter_compendium
            .validate_persisted()
            .map_err(GameError::InvalidStaticData)?;
        let abnormalities = self
            .game_data
            .abnormality_data
            .items
            .iter()
            .map(|metadata| {
                let registered = self
                    .state
                    .player_profile
                    .encounter_compendium
                    .registered_ids
                    .contains(&metadata.id);
                EncounterCompendiumAbnormalitySnapshotDto {
                    abnormality_id: metadata.id.clone(),
                    registered,
                    name: registered.then(|| metadata.name.clone()),
                    risk_level: registered.then_some(metadata.risk_level),
                }
            })
            .collect();
        let corrosion_profiles = self
            .game_data
            .corroded_employee_data
            .profiles
            .iter()
            .map(|metadata| {
                let registered = self
                    .state
                    .player_profile
                    .encounter_compendium
                    .registered_ids
                    .contains(&metadata.id);
                EncounterCompendiumCorrosionSnapshotDto {
                    profile_id: metadata.id.clone(),
                    registered,
                    name: registered.then(|| metadata.name.clone()),
                }
            })
            .collect();
        Ok(EncounterCompendiumSnapshotDto {
            abnormalities,
            corrosion_profiles,
        })
    }

    pub fn get_run_snapshot_json(&self) -> Result<Value, GameError> {
        crate::authoring::projection::to_value(self.get_run_snapshot_dto()?)
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))
    }

    pub(super) fn visual_novel_snapshot_dto(
        &self,
    ) -> Result<Option<VisualNovelSnapshotDto>, GameError> {
        let Some(session) = self.state.visual_novel.active.as_ref() else {
            return Ok(None);
        };
        let visual_novel = self
            .game_data
            .visual_novel_data
            .get_by_id(session.vn_id.as_str())
            .ok_or(GameError::MissingResource("VisualNovel"))?;
        let scene = visual_novel_scene(visual_novel, session.current_scene_id.as_str())?;
        let choices = match &scene.next {
            VisualNovelSceneNext::Choices { choices } => choices
                .iter()
                .map(|choice| VisualNovelChoiceSnapshotDto {
                    choice_id: choice.id.as_str().to_string(),
                    label_id: choice.label_id.clone(),
                    preview: visual_novel_choice_preview_dto(&choice.preview),
                    selectable: session.committed_choice_id.is_none()
                        || session.committed_choice_id.as_ref() == Some(&choice.id),
                })
                .collect(),
            _ => vec![],
        };
        Ok(Some(VisualNovelSnapshotDto {
            source: visual_novel_source_dto(&session.source),
            session_id: session.session_id.clone(),
            vn_id: session.vn_id.as_str().to_string(),
            current_scene_id: session.current_scene_id.as_str().to_string(),
            background_id: visual_novel_background_id(&scene.background),
            script_id: scene.script_id.clone(),
            speakers: scene
                .speakers
                .iter()
                .map(|speaker| VisualNovelSpeakerDto {
                    speaker_id: speaker.speaker_id.clone(),
                    portrait_id: speaker.portrait_id.clone(),
                    position: visual_novel_speaker_position_dto(speaker.position),
                    active: speaker.active,
                })
                .collect(),
            choices,
            blocking: session.blocking,
            presentation_mode: visual_novel_session_presentation_mode_dto(
                session.presentation_mode,
            ),
        }))
    }

    fn game_state_context_dto(&self, stress_combat: Option<crate::game::stress_combat::StressCombatView>) -> GameStateContextDto {
        match self.get_state() {
            GameState::NotStarted => GameStateContextDto::NotStarted,
            GameState::PreRunBriefing { game_mode } => {
                GameStateContextDto::PreRunBriefing { game_mode }
            }
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
                let combat_preview = run.filter(|_| !self.current_facility_encounter_hidden(node_id))
                    .and_then(|run| run.combat_previews.get(&node_id).cloned());
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
                    trauma_recovery_preview: if category == crate::game::map::MapNodeCategory::Gate && self.facility_layout().is_some() { None } else { self
                        .trauma_recovery_preview_for_category(category)
                        .ok()
                        .flatten() },
                    abnormality_attempt,
                    omen: run.and_then(|run| run.boss_omen.node_confirm_omen(node_id)),
                    monolith_conversion: self.monolith_conversion_snapshot(),
                    monolith_remanifestation: self.monolith_remanifestation_snapshot(),
                    mastered_elite_resolution: self.mastered_elite_resolution_dto(),
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
            GameState::MasteredEliteSweepResult { sweep_uuid } => {
                GameStateContextDto::MasteredEliteSweepResult { sweep_uuid }
            }
            GameState::InBattle { battle_uuid } => {
                let active = self
                    .state
                    .active_battle
                    .as_ref()
                    .expect("GameState::InBattle requires an active battle session");
                let abnormality_attempt = self
                    .state
                    .run
                    .as_ref()
                    .filter(|_| active.stress_combat.is_none())
                    .map(|run| run.abnormality_attempt_dto(active.node_id));
                let can_retreat = self.active_battle_retreat_available();
                GameStateContextDto::InBattle {
                    battle_uuid,
                    node_type: Some(active.node_type),
                    mission_variant: Some(active.mission_variant),
                    encounter_id: Some(active.encounter_id.clone()),
                    combat_preview: Some(active.combat_preview.clone()),
                    battle_state_revision: active.battle_state_revision,
                    tactical_resource: active.tactical_resource.to_dto(),
                    deployment: active.live_deployment_dto(active.employee_roster()),
                    playback: Some(active.playback_state()),
                    abnormality_attempt,
                    can_retreat,
                    stress_combat,
                }
            }
            GameState::AwaitingRunFailureDecision {
                reason,
                contributing_reason,
            } => GameStateContextDto::AwaitingRunFailureDecision {
                reason,
                contributing_reason,
            },
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
                    |(target, reach, target_range, air_capable)| SkillCatalogCastTargetDto {
                        selection: skill.cast_selection(),
                        target_policy: target.clone(),
                        reach,
                        target_range: target_range.cloned(),
                        air_capable,
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
                                crate::game::ability::DeliveryDef::TileArea { footprint, area } => {
                                    Some(SkillCatalogTileAreaDto {
                                        footprint: footprint.clone(),
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
            version: 3,
            range_source_of_truth: "range_previews.final_cells".to_string(),
            skills,
        }
    }

    pub(super) fn fragment_forms_snapshot(
        &self,
        copy: &crate::game::skill_fragment::OwnedFragment,
    ) -> Vec<crate::game::behavior::FragmentFormSnapshotDto> {
        let Some(metadata) = self
            .game_data
            .skill_fragment_data
            .get_by_id(&copy.definition_id)
        else {
            return vec![];
        };
        let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = &metadata.effect
        else {
            return vec![];
        };
        forms
            .iter()
            .map(|(id, form)| {
                let unlock_unavailable_reason =
                    self.fragment_form_unavailable_reason(copy.copy_uuid, id, false);
                let select_unavailable_reason =
                    self.fragment_form_unavailable_reason(copy.copy_uuid, id, true);
                crate::game::behavior::FragmentFormSnapshotDto {
                    form_id: id.clone(),
                    skill_id: form.skill_id.clone(),
                    resonance_max: form.resonance_max,
                    unlocked: copy.unlocked_forms.contains(id),
                    selected: copy.selected_form == *id,
                    can_unlock: unlock_unavailable_reason.is_none(),
                    can_select: select_unavailable_reason.is_none(),
                    unlock_unavailable_reason,
                    select_unavailable_reason,
                    recipe: form.bloom_recipe.clone(),
                }
            })
            .collect()
    }

    pub fn get_inventory_snapshot_dto(&self) -> Result<InventorySnapshotDto, GameError> {
        let inventory = self.inventory()?;
        let skill_fragments = self.state.skill_fragments.copies().map(|copy| {
            let metadata = self.game_data.skill_fragment_data.get_by_id(&copy.definition_id);
            let (dismantle_recipe, unavailable_reason) = match metadata.map(|m| &m.effect) {
                Some(crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { dismantle_recipe, .. }) => (dismantle_recipe.clone(), None),
                Some(_) => (None, Some("fragment_active_skill_unavailable".into())),
                None => (None, Some("fragment_definition_missing".into())),
            };
            InventorySkillFragmentSnapshotDto {
                copy: copy.clone(), definition_count: self.state.skill_fragments.count(&copy.definition_id),
                name: metadata.map(|m| m.name.clone()), effect: metadata.map(|m| wire_skill_fragment_effect_label(&m.effect).into()),
                forms: self.fragment_forms_snapshot(copy), dismantle_recipe, unavailable_reason,
            }
        }).collect::<Vec<_>>();

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

        let mut consumables = inventory
            .consumables
            .iter()
            .map(crate::game::resources::ConsumableItemDto::from_consumable_stack)
            .collect::<Vec<_>>();
        consumables.sort_by(|left, right| left.definition_id.cmp(&right.definition_id));

        let mut equipment_materials = inventory
            .equipment_materials
            .iter()
            .filter(|(id, _)| id.as_str() != "equipment_dust")
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
        if self.state.crafting_resources.equipment_dust > 0 {
            let metadata = self.game_data.equipment_data.get_material_by_id("equipment_dust");
            equipment_materials.push(InventoryEquipmentMaterialSnapshotDto {
                material_id: "equipment_dust".into(), amount: self.state.crafting_resources.equipment_dust,
                name: metadata.map(|m| m.name.clone()), description: metadata.map(|m| m.description.clone()),
                material_type: metadata.map(|m| wire_equipment_material_type_label(m.material_type).into()),
                rarity: metadata.map(|m| m.rarity), equipment_type: metadata.and_then(|m| m.equipment_type),
            });
        }
        equipment_materials.sort_by(|left, right| left.material_id.cmp(&right.material_id));

        Ok(InventorySnapshotDto {
            reserved_slot_count: inventory.reserved_slot_count(),
            occupied_slot_count: inventory.occupied_slot_count(),
            equipments,
            equipment_materials,
            consumables,
            skill_fragments,
            fragment_dust_usage: "reserved_legacy_asset_not_used_by_fragment_recipes".into(),
            fragment_dust: self.state.skill_fragments.fragment_dust(),
        })
    }

    pub fn get_inventory_snapshot_json(&self) -> Result<Value, GameError> {
        crate::authoring::projection::to_value(self.get_inventory_snapshot_dto()?)
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))
    }

    pub fn get_employee_roster_snapshot_dto(&self) -> Result<EmployeeRosterSnapshotDto, GameError> {
        let roster = self.roster()?;
        let inventory = self.inventory()?;

        let mut employees = roster
            .iter()
            .map(|employee| -> Result<EmployeeSnapshotDto, GameError> {
                let effective_profile_result = effective_combat_profile_for_employee_with_relics(
                    roster,
                    inventory,
                    &self.state.skill_fragments,
                    &self.state.relics,
                    &self.game_data,
                    employee.uuid,
                );
                let (effective_profile, effective_profile_error) = match effective_profile_result {
                    Ok(profile) => (Some(profile), None),
                    Err(error) => (None, Some(effective_profile_error_dto(&error))),
                };
                let skill_fragment_compatibility =
                    self.state
                        .skill_fragments
                        .copies()
                        .map(|copy| {
                            let metadata = self
                                .game_data
                                .skill_fragment_data
                                .get_by_id(&copy.definition_id);
                            let report = self.skill_fragment_compatibility_report_for_employee(
                                employee.uuid,
                                copy.copy_uuid,
                            );
                            let unavailable_reason = report.as_ref().err().map(|_| {
                                "fragment_active_skill_or_profile_unavailable".to_string()
                            });
                            EmployeeSkillFragmentCompatibilitySnapshotDto {
                                copy_uuid: copy.copy_uuid,
                                definition_id: copy.definition_id.clone(),
                                selected_form: copy.selected_form.clone(),
                                name: metadata.map(|m| m.name.clone()),
                                is_compatible: report.as_ref().is_ok_and(|r| r.is_compatible),
                                failure_codes: report.map(|r| r.failure_codes).unwrap_or_default(),
                                requirements: metadata.map(|m| m.compatibility.clone()),
                                unavailable_reason,
                            }
                        })
                        .collect::<Vec<_>>();
                Ok(EmployeeSnapshotDto {
                    role: employee.role,
                    uuid: employee.uuid,
                    employee_definition_id: employee.employee_definition_id.clone(),
                    name: employee.name.clone(),
                    profile: employee.profile.clone(),
                    level: employee.level,
                    experience: employee.experience,
                    condition: EmployeeConditionSnapshotDto {
                        breakdown_remaining_cost: employee.breakdown.remaining_cost(),
                        physical_damage: employee.condition.physical_damage,
                        physical_state: employee.condition.physical_state(),
                        trauma: employee.condition.trauma,
                        mental_state: employee.condition.mental_state(),
                    },
                    eligibility: EmployeeEligibilitySnapshotDto {
                        combat: employee
                            .eligibility(crate::game::employee::EmployeeActivity::Combat),
                        event: employee.eligibility(crate::game::employee::EmployeeActivity::Event),
                        high_risk: employee
                            .eligibility(crate::game::employee::EmployeeActivity::HighRisk),
                        consumable: employee
                            .eligibility(crate::game::employee::EmployeeActivity::Consumable),
                        loadout: employee
                            .eligibility(crate::game::employee::EmployeeActivity::Loadout),
                        physical_treatment: employee.eligibility(
                            crate::game::employee::EmployeeActivity::PhysicalTreatment,
                        ),
                        trauma_recovery: employee
                            .eligibility(crate::game::employee::EmployeeActivity::TraumaRecovery),
                    },
                    combat_profile: EmployeeCombatProfileSnapshotDto {
                        battle_tier: wire_tier_label(
                            employee.battle_tier(self.game_data.run_policy.as_ref()),
                        )
                        .into(),
                        base_stats: employee.combat_profile.battle_profile.stats,
                        basic_attack: employee.combat_profile.battle_profile.basic_attack.clone(),
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
                            .baseline_ids()
                            .iter()
                            .chain(
                                employee
                                    .skill_fragments
                                    .active_copy_uuid()
                                    .and_then(|id| self.state.skill_fragments.get(&id))
                                    .map(|copy| &copy.definition_id),
                            )
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
                        active_copy_uuid: employee.skill_fragments.active_copy_uuid(),
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
                                weapon_stage: equipment.map(|item| item.weapon_stage).unwrap_or_default(),
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
                })
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        employees.sort_by(|left, right| left.uuid.cmp(&right.uuid));

        Ok(EmployeeRosterSnapshotDto {
            employees,
            combat_eligible_employee_ids: roster.combat_eligible_employee_ids(),
        })
    }

    pub fn get_employee_roster_snapshot_json(&self) -> Result<Value, GameError> {
        crate::authoring::projection::to_value(self.get_employee_roster_snapshot_dto()?)
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))
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

    fn display_shop_offer_item_snapshot_dto(
        &self,
        offer: &crate::game::resources::ShopOfferState,
    ) -> Result<DisplayItemSnapshotDto, GameError> {
        use crate::game::data::shop_data::ShopMerchandise;
        use crate::game::behavior::{DisplayRelicItemSnapshotDto, DisplayCatalystItemSnapshotDto, ShopCatalystPurpose};
        match &offer.merchandise {
            ShopMerchandise::Relic { relic_id } => {
                let definition = self.game_data.relic_data.get_by_id(relic_id).ok_or(GameError::InvalidAction)?;
                return Ok(DisplayItemSnapshotDto::Relic(DisplayRelicItemSnapshotDto {
                    uuid: definition.uuid, kind: "relic".into(), id: relic_id.clone(), definition_id: relic_id.clone(),
                    relic: definition.clone(), variant: crate::game::relic::RelicVariant::Normal,
                }));
            }
            ShopMerchandise::RevivalCatalyst { material_id } => {
                let definition = self.game_data.equipment_data.get_material_by_id(material_id).ok_or(GameError::InvalidAction)?;
                return Ok(DisplayItemSnapshotDto::Catalyst(DisplayCatalystItemSnapshotDto {
                    uuid: definition.uuid, kind: "catalyst".into(), id: material_id.clone(), definition_id: material_id.clone(),
                    name: definition.name.clone(), description: definition.description.clone(), purpose: ShopCatalystPurpose::Revival,
                }));
            }
            ShopMerchandise::RelicCorruptionCatalyst => return Ok(DisplayItemSnapshotDto::Catalyst(DisplayCatalystItemSnapshotDto {
                uuid: offer.item_uuid, kind: "catalyst".into(), id: "relic_corruption_catalyst".into(), definition_id: "relic_corruption_catalyst".into(),
                name: "침식 촉매제".into(), description: "보유 유물 하나를 침식합니다. 환상체 부활에는 사용할 수 없습니다.".into(), purpose: ShopCatalystPurpose::RelicCorruption,
            })),
            _ => {}
        }
        if matches!(offer.merchandise, ShopMerchandise::Monolith { .. }) {
            return Err(Self::retired_encounter_action(crate::game::behavior::ActionKind::PurchaseItem));
        }
        self.display_item_snapshot_dto(offer.item_uuid)
    }

    pub fn get_selected_event_snapshot_dto(
        &self,
    ) -> Result<Option<SelectedEventSnapshotDto>, GameError> {
        if let Some(receipt) = self.state.confirmed_combat_result.as_ref() {
            return Ok(Some(receipt.summary.clone()));
        }
        if matches!(
            self.state.lifecycle.phase(),
            GameState::MasteredEliteSweepResult { .. }
        ) {
            let result = self
                .staged_mastered_elite_sweep_result()
                .ok_or(GameError::InvalidAction)?;
            return Ok(Some(SelectedEventSnapshotDto::MasteredEliteSweepResult {
                result: result.clone(),
            }));
        }
        let Some(selected) = self.state.active_node_content.as_ref() else {
            return Ok(None);
        };

        let value = match selected {
            ActiveNodeContent::Shop(shop) => SelectedEventSnapshotDto::Shop {
                id: shop.id.clone(),
                name: shop.name.clone(),
                uuid: shop.uuid,
                context: shop.context,
                offers: shop
                    .offers
                    .iter()
                    .map(|offer| {
                        Ok(crate::game::behavior::ShopOfferSnapshotDto {
                            offer_id: offer.offer_id,
                            merchandise: offer.merchandise.clone(),
                            quantity: offer.quantity,
                            total_price: offer.total_price,
                            item: self.display_shop_offer_item_snapshot_dto(offer)?,
                            eligibility: self.shop_purchase_eligibility(offer),
                        })
                    })
                    .collect::<Result<Vec<_>, GameError>>()?,
                refresh: {
                    let eligibility = self.shop_refresh_eligibility(shop);
                    match shop.refresh_policy {
                        Some(policy) => crate::game::behavior::ShopRefreshSnapshotDto {
                            available: true,
                            current_cost: Some(policy.base_cost),
                            max_uses: policy.max_uses,
                            uses_consumed: shop.refresh_uses,
                            remaining_uses: policy.max_uses.saturating_sub(shop.refresh_uses),
                            eligibility,
                        },
                        None => crate::game::behavior::ShopRefreshSnapshotDto {
                            available: false,
                            current_cost: None,
                            max_uses: 0,
                            uses_consumed: 0,
                            remaining_uses: 0,
                            eligibility,
                        },
                    }
                },
                selling_enabled: shop.sell_policy.is_some(),
                sale_candidates: self.shop_sale_candidates(shop),
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
                relic_activity: self.relic_reward_view(reward)?,
                dealer: self.dealer_view(reward)?,
            },
            ActiveNodeContent::Event(event) => SelectedEventSnapshotDto::Event {
                node_id: event.node_id,
                event: self.event_scene_snapshot(event)?,
            },
            ActiveNodeContent::SavePoint(save_point) => SelectedEventSnapshotDto::SavePoint {
                node_id: save_point.node_id,
                recovery_preview: self
                    .trauma_recovery_preview_for_category(
                        crate::game::map::MapNodeCategory::SavePoint,
                    )?
                    .ok_or(GameError::InvalidAction)?,
            },
            ActiveNodeContent::Maintenance(maintenance) => SelectedEventSnapshotDto::Maintenance {
                node_id: maintenance.node_id,
                maintenance_options: self.maintenance_options(),
            },
            ActiveNodeContent::HeadquartersContact(headquarters) => {
                SelectedEventSnapshotDto::HeadquartersContact {
                    node_id: headquarters.node_id,
                    options: headquarters.options.clone(),
                    recruitment_candidates: self.active_recruitment_candidates()?,
                    recruitment_terms: self.headquarters_recruitment_terms()?,
                    shop_pool_id: headquarters.shop_pool_id.clone(),
                }
            }
            ActiveNodeContent::CombatBattle(battle) => SelectedEventSnapshotDto::CombatBattle {
                primary_abnormality_id: battle.primary_abnormality_id.clone(),
                encounter_id: battle.encounter_id.clone(),
                node_type: battle.node_type,
                mission_variant: battle.mission_variant,
                abnormality_uuid: battle.abnormality_uuid,
                outcome_id: battle.outcome_id.clone(),
                verdict: battle.verdict,
                winner: battle.winner,
                reward_mode: battle.reward_mode,
                rewards: battle
                    .rewards
                    .iter()
                    .map(display_reward_option_snapshot_dto)
                    .collect::<Vec<_>>(),
                result_stats: battle.result_stats.clone(),
                no_reward_candidates: battle.reward_mode
                    == crate::game::enums::RewardMode::ChooseOne
                    && battle.rewards.is_empty(),
                has_event_log: COMBAT_TRACE_CAPABILITY_AVAILABLE,
            },
        };

        Ok(Some(value))
    }

    pub fn get_selected_event_snapshot_json(&self) -> Result<Option<Value>, GameError> {
        self.get_selected_event_snapshot_dto()?
            .map(|selected| {
                crate::authoring::projection::to_value(selected).map_err(|error| {
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
    use crate::game::behavior::{ActionDisabledReasonCode, ActionKind, BattleStateAuthorityDto};
    use crate::game::data::skill_fragment_data::{
        SkillFragmentCompatibilityFailureCode, SkillFragmentId,
    };

    const ALL_DISABLED_REASONS: [ActionDisabledReasonCode; 41] = [
        ActionDisabledReasonCode::ContextIrrelevant,
        ActionDisabledReasonCode::NoEligiblePayload,
        ActionDisabledReasonCode::MissingEventChoiceKey,
        ActionDisabledReasonCode::CheckpointUnavailable,
        ActionDisabledReasonCode::ProgressionUnavailable,
        ActionDisabledReasonCode::ContentUnavailable,
        ActionDisabledReasonCode::CompletionUnavailable,
        ActionDisabledReasonCode::AlreadyActive,
        ActionDisabledReasonCode::InsufficientResources,
        ActionDisabledReasonCode::RedeployCooldown,
        ActionDisabledReasonCode::StanceCooldown,
        ActionDisabledReasonCode::RelocationCooldown,
        ActionDisabledReasonCode::UnitUnavailable,
        ActionDisabledReasonCode::WeaponRequired,
        ActionDisabledReasonCode::EmployeeDead,
        ActionDisabledReasonCode::CriticalPhysicalDamage,
        ActionDisabledReasonCode::Breakdown,
        ActionDisabledReasonCode::NoTrauma,
        ActionDisabledReasonCode::SkillUnavailable,
        ActionDisabledReasonCode::ActivationModeNotManual,
        ActionDisabledReasonCode::HardCrowdControl,
        ActionDisabledReasonCode::Silenced,
        ActionDisabledReasonCode::DefenseStance,
        ActionDisabledReasonCode::DefenseRequired,
        ActionDisabledReasonCode::ActionInProgress,
        ActionDisabledReasonCode::ActionLocked,
        ActionDisabledReasonCode::FieldWorking,
        ActionDisabledReasonCode::ResonanceNotFull,
        ActionDisabledReasonCode::SkillTargetRequired,
        ActionDisabledReasonCode::SkillTargetTypeMismatch,
        ActionDisabledReasonCode::NoValidTarget,
        ActionDisabledReasonCode::OfferUnavailable,
        ActionDisabledReasonCode::RefreshUnavailable,
        ActionDisabledReasonCode::RefreshUsesExhausted,
        ActionDisabledReasonCode::PoolExhausted,
        ActionDisabledReasonCode::SellingUnavailable,
        ActionDisabledReasonCode::InvalidQuantity,
        ActionDisabledReasonCode::InvalidQuote,
        ActionDisabledReasonCode::ItemBound,
        ActionDisabledReasonCode::ItemEquipped,
        ActionDisabledReasonCode::TagNotAccepted,
    ];

    fn authority(battle_uuid: u128, battle_state_revision: u64) -> BattleStateAuthorityDto {
        BattleStateAuthorityDto {
            battle_uuid: Uuid::from_u128(battle_uuid),
            battle_state_revision,
        }
    }

    fn characterized_fixed_game_errors() -> Vec<(GameError, &'static str, String)> {
        let current = authority(0xCAFE, 17);
        let expected = authority(0xBEEF, 11);

        vec![
            (
                GameError::EventNotFound,
                "event_not_found",
                "event not found".into(),
            ),
            (
                GameError::EventTypeMismatch,
                "event_type_mismatch",
                "event type mismatch".into(),
            ),
            (
                GameError::InvalidAction,
                "invalid_action",
                "invalid action".into(),
            ),
            (
                GameError::WeaponRequired,
                "weapon_required",
                "weapon required".into(),
            ),
            (
                GameError::InvalidBattleResyncSeq {
                    requested: 29,
                    latest: 23,
                },
                "invalid_battle_resync_seq",
                "invalid battle resync seq: requested 29, latest 23".into(),
            ),
            (
                GameError::BattleSnapshotResyncRequired {
                    requested: 3,
                    oldest_available_after_seq: 8,
                    latest: 23,
                },
                "battle_snapshot_resync_required",
                "battle snapshot resync required: requested 3, oldest available after seq 8, \
                 latest 23"
                    .into(),
            ),
            (
                GameError::BattleStateRevisionOverflow,
                "battle_state_revision_overflow",
                "battle state revision overflow".into(),
            ),
            (
                GameError::MissingBattleStateAuthority { current },
                "missing_battle_state_authority",
                format!(
                    "missing battle state authority: current {}@{}",
                    current.battle_uuid, current.battle_state_revision
                ),
            ),
            (
                GameError::StaleBattleStateAuthority { expected, current },
                "stale_battle_state_authority",
                format!(
                    "stale battle state authority: expected {}@{}, current {}@{}",
                    expected.battle_uuid,
                    expected.battle_state_revision,
                    current.battle_uuid,
                    current.battle_state_revision
                ),
            ),
            (
                GameError::NotInShopState,
                "not_in_shop_state",
                "not in shop state".into(),
            ),
            (
                GameError::NotInRewardState,
                "not_in_reward_state",
                "not in reward state".into(),
            ),
            (
                GameError::ShopItemNotFound,
                "shop_item_not_found",
                "shop item not found".into(),
            ),
            (
                GameError::InventoryFull,
                "inventory_full",
                "inventory full".into(),
            ),
            (
                GameError::InventoryItemNotFound,
                "inventory_item_not_found",
                "inventory item not found".into(),
            ),
            (
                GameError::InventoryItemNotRemovable,
                "inventory_item_not_removable",
                "inventory item not removable".into(),
            ),
            (
                GameError::InsufficientResources,
                "insufficient_resources",
                "insufficient resources".into(),
            ),
            (
                GameError::MissingResource("RunState"),
                "missing_resource",
                "missing resource: RunState".into(),
            ),
            (
                GameError::InvalidUnitStats("negative defense"),
                "invalid_unit_stats",
                "invalid unit stats: negative defense".into(),
            ),
            (
                GameError::InvalidStaticData("broken fixture".into()),
                "invalid_static_data",
                "invalid static data: broken fixture".into(),
            ),
            (
                GameError::NotImplemented("future loop"),
                "not_implemented",
                "not implemented: future loop".into(),
            ),
            (
                GameError::SkillFragmentIncompatible {
                    fragment_id: SkillFragmentId::from("fragment_contract"),
                    failure_codes: vec![
                        SkillFragmentCompatibilityFailureCode::WeaponRequired,
                        SkillFragmentCompatibilityFailureCode::RangeRoleMismatch,
                    ],
                },
                "skill_fragment_incompatible",
                "skill fragment incompatible: fragment_contract; failure code count 2".into(),
            ),
            (
                GameError::OutOfBounds,
                "out_of_bounds",
                "out of bounds".into(),
            ),
            (
                GameError::PositionOccupied,
                "position_occupied",
                "position occupied".into(),
            ),
            (
                GameError::StaticObstacleBlocked,
                "static_obstacle_blocked",
                "static obstacle blocked".into(),
            ),
            (
                GameError::UnitAlreadyPlaced,
                "unit_already_placed",
                "unit already placed".into(),
            ),
            (
                GameError::UnitNotFound,
                "unit_not_found",
                "unit not found".into(),
            ),
        ]
    }

    #[test]
    fn snapshot_error_projection_characterizes_every_game_error_code_and_message() {
        let fixed_cases = characterized_fixed_game_errors();
        assert_eq!(fixed_cases.len(), 26);
        assert_eq!(ALL_DISABLED_REASONS.len(), 41);

        for (error, expected_code, expected_message) in fixed_cases {
            let actual = effective_profile_error_dto(&error);
            assert_eq!(actual.code, expected_code, "{error:?}");
            assert_eq!(actual.message, expected_message, "{error:?}");
            assert_eq!(
                serde_json::to_value(&actual).unwrap(),
                serde_json::json!({
                    "code": expected_code,
                    "message": expected_message,
                }),
                "{error:?}"
            );
        }

        for reason in ALL_DISABLED_REASONS {
            let error = GameError::ActionUnavailable {
                action: ActionKind::DeployUnit,
                reason,
            };
            let expected_message = format!("action unavailable: DeployUnit ({reason:?})");
            let actual = effective_profile_error_dto(&error);
            assert_eq!(actual.code, reason.as_str(), "{reason:?}");
            assert_eq!(actual.message, expected_message, "{reason:?}");
            assert_eq!(
                serde_json::to_value(&actual).unwrap(),
                serde_json::json!({
                    "code": reason.as_str(),
                    "message": expected_message,
                }),
                "{reason:?}"
            );
        }

        assert_eq!(25 + ALL_DISABLED_REASONS.len(), 66);
    }

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
        assert_eq!(wire_tier_label(crate::game::enums::Tier::III), "III");
    }
}

impl GameCore {
    pub fn certified_input_fingerprint(
        &self,
    ) -> Result<crate::game::combat_power::CertifiedInputFingerprint, GameError> {
        crate::game::combat_power::certified_input_fingerprint_for_roster(
            &self.state.roster,
            &self.state.inventory,
            &self.state.skill_fragments,
            &self.game_data,
        )
    }
}
