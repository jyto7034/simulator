use super::facility::StagedRecoveryEffect;
use super::state::{RunCheckpointPayload, RUN_CHECKPOINT_MAX_LOADS};
use super::{map_encounters, GameCore, RunState};
use crate::game::abnormality_research::{
    RunAbnormalityEncounterHistory, RunAbnormalityResearchState,
};
use crate::game::behavior::{
    BehaviorResult, GameError, MonolithConversionOptionDto, MonolithConversionSnapshotDto,
    MonolithRemanifestationSnapshotDto, MonolithRewardTierDto,
};
use crate::game::boss_omen::{
    apply_boss_omen_to_map, complete_forced_boss_with_exit_gate, force_boss_node_if_ready,
    BossOmenRunState,
};
use crate::game::map::{
    FacilityGenerationRequirements, FacilityMapGenerator, FacilityRunState,
    FacilityTerminalResolution, MapNodeCategory, MapNodeCombatDifficultyClass, MapNodeExecutor,
    MapNodeId, MapNodePayload, MapNodeState, MapProgression, MapViewDto,
    MonolithRemanifestationStage, RunMap, RunProgression,
};
use crate::game::resources::{ActiveNodeContent, GameState};
use crate::game::skill_fragment::{SkillFragmentInventory, SkillFragmentResearchDelivery};
use tracing::info;

fn percent_of_anchor_ceil(anchor: u64, percent: u32) -> u64 {
    let value = u128::from(anchor)
        .saturating_mul(u128::from(percent))
        .div_ceil(100);
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn validate_monolith_difficulty_authority(
    node_id: MapNodeId,
    difficulty: Option<&crate::game::map::MapNodeCombatDifficulty>,
    monolith: &crate::game::monolith::MonolithRunState,
) -> Result<(), GameError> {
    let ledger = monolith.remanifestation(node_id);
    let is_monolith_role = difficulty.is_some_and(|difficulty| {
        matches!(
            difficulty.class,
            MapNodeCombatDifficultyClass::Role(
                crate::game::data::run_policy_data::CombatNodeDifficultyRole::MonolithRemanifestation
            )
        )
    });
    let typed_stage = difficulty.and_then(|difficulty| difficulty.monolith_stage);
    match (is_monolith_role, typed_stage, ledger) {
        (false, None, None) => Ok(()),
        (true, Some(stage), Some(ledger)) if stage.as_u8() == ledger.stage => Ok(()),
        _ => Err(GameError::InvalidStaticData(format!(
            "monolith difficulty authority for node '{}' does not match its ledger",
            node_id.0
        ))),
    }
}

pub(super) enum StagedNodeCompletion {
    NodeCompleted {
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        boss_omen: Option<BossOmenRunState>,
        recovery_effect: StagedRecoveryEffect,
        view: MapViewDto,
    },
    FloorAdvanced {
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        facility_run_state: FacilityRunState,
        boss_omen: BossOmenRunState,
        recovery_effect: StagedRecoveryEffect,
        view: MapViewDto,
    },
    RunComplete {
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        recovery_effect: StagedRecoveryEffect,
        view: MapViewDto,
    },
}

pub(super) enum CombatResultNodeCompletion {
    NodeCompleted {
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        boss_omen: Option<BossOmenRunState>,
        view: MapViewDto,
    },
    FloorAdvanced {
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        facility_run_state: FacilityRunState,
        boss_omen: BossOmenRunState,
        view: MapViewDto,
    },
    RunComplete {
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
    },
}

impl TryFrom<StagedNodeCompletion> for CombatResultNodeCompletion {
    type Error = GameError;

    fn try_from(completion: StagedNodeCompletion) -> Result<Self, Self::Error> {
        match completion {
            StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen,
                recovery_effect,
                view,
            } => {
                if recovery_effect != StagedRecoveryEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::NodeCompleted {
                    map,
                    progression,
                    completing_node_id,
                    boss_omen,
                    view,
                })
            }
            StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                boss_omen,
                recovery_effect,
                view,
            } => {
                if recovery_effect != StagedRecoveryEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::FloorAdvanced {
                    next_map,
                    next_progression,
                    run_progression,
                    facility_run_state,
                    boss_omen,
                    view,
                })
            }
            StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                recovery_effect,
                view,
            } => {
                if recovery_effect != StagedRecoveryEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::RunComplete {
                    map,
                    progression,
                    run_progression,
                    view,
                })
            }
        }
    }
}

impl GameCore {
    pub(super) fn facility_template_for_map(
        &self,
        map: &RunMap,
    ) -> Result<crate::game::map::FacilityTemplate, GameError> {
        let template = self
            .game_data
            .map_catalogs
            .facility_templates
            .template(&map.map_template_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "run map references missing facility template '{}'",
                    map.map_template_id.as_str()
                ))
            })?;
        if template.version != map.facility_template_version {
            return Err(GameError::InvalidStaticData(format!(
                "run map template '{}@{}' does not match live template version {}",
                map.map_template_id.as_str(),
                map.facility_template_version,
                template.version
            )));
        }
        Ok(template.clone())
    }

    pub(super) fn deliver_pending_research_in_staged_inventory(
        &self,
        skill_fragments: &mut SkillFragmentInventory,
    ) -> Result<Vec<SkillFragmentResearchDelivery>, GameError> {
        skill_fragments.deliver_pending_research_with_policy(
            &self.game_data.skill_fragment_data,
            &self.state.skill_fragment_policy,
        )
    }

    fn stage_current_planning_return_research_delivery(
        &self,
    ) -> Result<(SkillFragmentInventory, Vec<SkillFragmentResearchDelivery>), GameError> {
        let mut skill_fragments = self.state.skill_fragments.clone();
        let deliveries = self.deliver_pending_research_in_staged_inventory(&mut skill_fragments)?;
        Ok((skill_fragments, deliveries))
    }

    fn log_planning_return_research_deliveries(deliveries: &[SkillFragmentResearchDelivery]) {
        if !deliveries.is_empty() {
            info!(
                "Delivered pending skill fragment research on authoritative planning return: {:?}",
                deliveries
            );
        }
    }

    pub(super) fn current_map_view(&self) -> Result<MapViewDto, GameError> {
        let run = self.run_state()?;
        let template = self.facility_template_for_map(&run.map)?;
        let mut view = run.map_progression.view(&run.map, &template);
        let certified = crate::game::combat_power::certified_combat_power_for_roster(
            &self.state.roster,
            &self.state.inventory,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
        )?;
        let threat_policy = self.run_policy().combat_difficulty.threat_bands;
        let mut readiness_warnings = Vec::new();
        if !self
            .state
            .roster
            .iter()
            .any(|employee| employee.is_combat_eligible())
        {
            readiness_warnings
                .push(crate::game::map::CombatReadinessWarningDto::NoDeployableEmployees);
        }
        if self
            .state
            .roster
            .iter()
            .any(|employee| employee.condition.is_breakdown())
        {
            readiness_warnings.push(crate::game::map::CombatReadinessWarningDto::TraumaBreakdown);
        }
        if self.state.roster.iter().any(|employee| {
            employee.condition.physical_state()
                == crate::game::employee::PhysicalDamageState::Critical
        }) {
            readiness_warnings
                .push(crate::game::map::CombatReadinessWarningDto::CriticalPhysicalDamage);
        }
        for node in &mut view.nodes {
            let Some(target_power) = run
                .map
                .node(node.id)
                .and_then(|source| source.combat_difficulty.as_ref())
                .map(|difficulty| difficulty.target_power)
            else {
                continue;
            };
            let ratio = if certified.total == 0 {
                u64::MAX
            } else {
                u64::try_from(
                    u128::from(target_power)
                        .saturating_mul(100)
                        .div_ceil(u128::from(certified.total)),
                )
                .unwrap_or(u64::MAX)
            };
            node.relative_threat =
                Some(if ratio <= u64::from(threat_policy.low_max_ratio_percent) {
                    crate::game::map::RelativeThreatBandDto::Low
                } else if ratio <= u64::from(threat_policy.equal_max_ratio_percent) {
                    crate::game::map::RelativeThreatBandDto::Equal
                } else if ratio <= u64::from(threat_policy.high_max_ratio_percent) {
                    crate::game::map::RelativeThreatBandDto::High
                } else {
                    crate::game::map::RelativeThreatBandDto::VeryHigh
                });
            node.readiness_warnings = readiness_warnings.clone();
        }
        if let Some(restriction) = run.combat_recovery_restriction.as_ref() {
            for node in &mut view.nodes {
                if node.state == crate::game::map::MapNodeState::Available
                    && !restriction.allowed_node_ids.contains(&node.id)
                {
                    node.state = crate::game::map::MapNodeState::Unavailable;
                }
            }
        }
        Ok(view)
    }

    pub(super) fn generate_current_floor_map(
        &mut self,
        run_progression: &RunProgression,
        facility_run_state: &FacilityRunState,
        abnormality_research: &RunAbnormalityResearchState,
        abnormality_encounter_history: &RunAbnormalityEncounterHistory,
        monolith: &crate::game::monolith::MonolithRunState,
        boss_omen: &mut crate::game::boss_omen::BossOmenRunState,
    ) -> (RunMap, MapProgression, FacilityRunState) {
        let power_model_version = self.run_policy().combat_difficulty.power_model_version;
        if self.state.run_power_authority.power_model_version == 0 {
            let certified = crate::game::combat_power::certified_combat_power_for_roster(
                &self.state.roster,
                &self.state.inventory,
                &self.state.skill_fragments,
                self.game_data.as_ref(),
            )
            .expect("validated persistent loadout state must certify");
            self.state
                .run_power_authority
                .initialize_run(power_model_version, certified.total);
        }
        let anchor_policy = self
            .run_policy()
            .floor_power_anchor_policy(run_progression.game_mode, run_progression.floor_index())
            .expect("validated run policy must resolve current floor power anchor");
        self.state.run_power_authority.manifest_floor(anchor_policy);
        let generated = FacilityMapGenerator::generate_with_requirements(
            run_progression.run_seed,
            run_progression.game_mode,
            run_progression.floor_index(),
            facility_run_state,
            &self.game_data.map_catalogs,
            self.run_policy(),
            self.state.run_power_authority.current_floor_power_anchor,
            FacilityGenerationRequirements {
                guarantee_reachable_shop: monolith.next_floor_shop_guarantee_pending,
            },
        )
        .expect("validated live facility data must generate the current Floor");
        let mut run_map = generated.map;
        map_encounters::assign_map_encounters(
            &self.game_data.pve_data,
            &mut run_map,
            run_progression,
            abnormality_research,
            abnormality_encounter_history,
            monolith,
            self.run_policy(),
        );
        let mut progression = MapProgression::from_map(&run_map);
        force_boss_node_if_ready(
            &mut run_map,
            &mut progression,
            run_progression,
            self.game_data.as_ref(),
            boss_omen,
        );
        if boss_omen.forced_boss_node_id().is_none() {
            apply_boss_omen_to_map(
                &mut run_map,
                run_progression,
                &self.state.skill_fragments,
                self.game_data.as_ref(),
                boss_omen,
            );
            progression = MapProgression::from_map(&run_map);
        }
        (run_map, progression, generated.facility_run_state)
    }

    pub(super) fn handle_request_map_data(&self) -> Result<BehaviorResult, GameError> {
        Ok(BehaviorResult::MapState {
            map: self.current_map_view()?,
        })
    }

    pub(super) fn handle_select_map_node(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_select_map_node(node_id)?;
        let run = self.run_state()?;
        let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
        let enter_result = MapNodeExecutor::enter(node);
        let combat_preview = self.combat_preview_for_node(node_id)?;
        let trauma_recovery_preview =
            self.trauma_recovery_preview_for_category(enter_result.category)?;
        self.state.node_session = Some(enter_result.session.clone());
        self.transition_to(GameState::NodeConfirm {
            node_id,
            kind_id: enter_result.kind_id.clone(),
            category: enter_result.category,
        })?;

        Ok(BehaviorResult::NodePreview {
            node_id: enter_result.node_id,
            kind_id: enter_result.kind_id,
            category: enter_result.category,
            payload: enter_result.payload,
            session: enter_result.session,
            map: self.current_map_view()?,
            combat_preview,
            trauma_recovery_preview,
        })
    }

    pub(super) fn handle_cancel_selected_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.state.node_session = None;
        self.transition_to(GameState::ViewingMap)?;
        self.handle_request_map_data()
    }

    pub(super) fn monolith_conversion_snapshot(&self) -> Option<MonolithConversionSnapshotDto> {
        let GameState::NodeConfirm { node_id, .. } = self.state.game_state else {
            return None;
        };
        let run = self.state.run.as_ref()?;
        if run.run_progression.game_mode != crate::game::map::GameMode::Endless {
            return None;
        }
        let node = run.map.node(node_id)?;
        if node.state != MapNodeState::Available
            || node.category != MapNodeCategory::Combat
            || node.kind_id.as_str().contains("elite")
            || node.omen.is_some()
            || run.monolith.remanifestation(node_id).is_some()
        {
            return None;
        }
        let MapNodePayload::Encounter {
            encounter_id: Some(encounter_id),
        } = &node.payload
        else {
            return None;
        };
        if self
            .game_data
            .pve_data
            .encounters
            .iter()
            .find(|encounter| encounter.id == *encounter_id)
            .is_none_or(|encounter| {
                encounter.encounter_class != crate::game::data::pve_data::PveEncounterClass::Normal
            })
        {
            return None;
        }

        let policy = &self.run_policy().monolith;
        let mut options = run
            .monolith
            .progress_by_abnormality
            .iter()
            .filter(|(_, progress)| progress.next_stage().is_some() && progress.uses_remaining > 0)
            .filter_map(|(abnormality_id, progress)| {
                let metadata = self.game_data.abnormality_data.get_by_id(abnormality_id)?;
                self.monolith_elite_encounter_id(abnormality_id)?;
                let next_stage = progress.next_stage()?;
                let disabled_reason = progress.active_node_id.and_then(|active_node_id| {
                    run.map
                        .node(active_node_id)
                        .map(|active_node| format!("진행 중: {}", active_node.slot_id.as_str()))
                });
                Some(MonolithConversionOptionDto {
                    abnormality_id: abnormality_id.clone(),
                    abnormality_uuid: metadata.uuid,
                    display_name: metadata.name.clone(),
                    risk_level: metadata.risk_level,
                    enabled: progress.is_usable(),
                    disabled_reason,
                    next_stage,
                    uses_before: progress.uses_remaining,
                    uses_after: progress.uses_remaining.saturating_sub(1),
                    max_attempts: policy.remanifestation_max_attempts,
                    reward_tier: match next_stage {
                        2 => MonolithRewardTierDto::Enhanced,
                        3 => MonolithRewardTierDto::FinalEnhanced,
                        _ => return None,
                    },
                })
            })
            .collect::<Vec<_>>();
        options.sort_by(|left, right| {
            right
                .next_stage
                .cmp(&left.next_stage)
                .then_with(|| left.display_name.cmp(&right.display_name))
                .then_with(|| left.abnormality_id.cmp(&right.abnormality_id))
        });
        let enabled_count = options.iter().filter(|option| option.enabled).count();
        (enabled_count > 0).then(|| MonolithConversionSnapshotDto {
            action_label: format!("모노리스 사용 ({enabled_count})"),
            node_id,
            options,
        })
    }

    pub(super) fn monolith_remanifestation_snapshot(
        &self,
    ) -> Option<MonolithRemanifestationSnapshotDto> {
        let GameState::NodeConfirm { node_id, .. } = self.state.game_state else {
            return None;
        };
        let run = self.state.run.as_ref()?;
        let remanifestation = run.monolith.remanifestation(node_id)?;
        let metadata = self
            .game_data
            .abnormality_data
            .get_by_id(&remanifestation.abnormality_id)?;
        let policy = &self.run_policy().monolith;
        let stage = MonolithRemanifestationStage::try_from(remanifestation.stage).ok()?;
        Some(MonolithRemanifestationSnapshotDto {
            abnormality_id: remanifestation.abnormality_id.clone(),
            abnormality_uuid: metadata.uuid,
            display_name: metadata.name.clone(),
            risk_level: metadata.risk_level,
            stage: remanifestation.stage,
            max_attempts: policy.remanifestation_max_attempts,
            reward_tier: match stage {
                MonolithRemanifestationStage::Stage2 => MonolithRewardTierDto::Enhanced,
                MonolithRemanifestationStage::Stage3 => MonolithRewardTierDto::FinalEnhanced,
            },
        })
    }

    fn monolith_elite_encounter_id(&self, abnormality_id: &str) -> Option<String> {
        self.game_data
            .pve_data
            .encounters
            .iter()
            .filter(|encounter| {
                encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::Elite
                    && encounter.primary_abnormality_id() == Some(abnormality_id)
            })
            .map(|encounter| encounter.id.clone())
            .min()
    }

    pub(super) fn handle_convert_map_node_with_monolith(
        &mut self,
        node_id: MapNodeId,
        abnormality_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(|core| {
            let projection = core
                .monolith_conversion_snapshot()
                .ok_or(GameError::InvalidAction)?;
            if projection.node_id != node_id
                || !projection
                    .options
                    .iter()
                    .any(|option| option.enabled && option.abnormality_id == abnormality_id)
            {
                return Err(GameError::InvalidAction);
            }
            let encounter_id = core
                .monolith_elite_encounter_id(abnormality_id)
                .ok_or(GameError::InvalidAction)?;
            let mut map = core.run_state()?.map.clone();
            let mut monolith = core.run_state()?.monolith.clone();
            let remanifestation = monolith
                .begin_remanifestation(node_id, abnormality_id, &encounter_id)
                .map_err(|_| GameError::InvalidAction)?;
            let node = map.node_mut(node_id).ok_or(GameError::InvalidAction)?;
            node.payload = MapNodePayload::Encounter {
                encounter_id: Some(encounter_id),
            };
            let difficulty = node
                .combat_difficulty
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
            difficulty.class = MapNodeCombatDifficultyClass::Role(
                crate::game::data::run_policy_data::CombatNodeDifficultyRole::MonolithRemanifestation,
            );
            difficulty.monolith_stage = Some(
                MonolithRemanifestationStage::try_from(remanifestation.stage)
                    .map_err(|_| GameError::InvalidAction)?,
            );
            let tuning = core
                .run_policy()
                .resolved_tuning_for_manifested_node(difficulty)
                .ok_or(GameError::InvalidAction)?;
            difficulty.target_power = percent_of_anchor_ceil(
                difficulty.floor_power_anchor,
                tuning.target_power_percent,
            );
            let enter_result = MapNodeExecutor::enter(node);
            {
                let run = core.run_state_mut()?;
                run.map = map;
                run.monolith = monolith;
                run.combat_previews.remove(&node_id);
                run.abnormality_attempts.remove(&node_id);
            }
            core.state.node_session = Some(enter_result.session.clone());
            core.transition_to(GameState::NodeConfirm {
                node_id,
                kind_id: enter_result.kind_id.clone(),
                category: enter_result.category,
            })?;
            let combat_preview = core.combat_preview_for_node(node_id)?;
            Ok(BehaviorResult::NodePreview {
                node_id,
                kind_id: enter_result.kind_id,
                category: enter_result.category,
                payload: enter_result.payload,
                session: enter_result.session,
                map: core.current_map_view()?,
                combat_preview,
                trauma_recovery_preview: None,
            })
        })
    }

    pub(super) fn handle_confirm_enter_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_confirm_enter_node_inner)
    }

    fn handle_confirm_enter_node_inner(&mut self) -> Result<BehaviorResult, GameError> {
        let node_id = match self.get_state() {
            GameState::NodeConfirm { node_id, .. } => node_id,
            _ => return Err(GameError::InvalidAction),
        };
        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let node = map.node(node_id).ok_or(GameError::InvalidAction)?;
        if matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        ) {
            if let Some(result) = self.block_or_fail_undeployable_combat_selection()? {
                return Ok(result);
            }
        }
        let battle_entry_snapshot = matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        )
        .then(|| super::state::BattleEntrySnapshot::capture(&self.state));

        let template = self.facility_template_for_map(&map)?;
        progression
            .enter_node(&mut map, &template, node_id)
            .map_err(|_| GameError::InvalidAction)?;
        let node = map.node(node_id).ok_or(GameError::InvalidAction)?;
        let enter_result = MapNodeExecutor::enter(node);

        if let Some(run) = self.state.run.as_mut() {
            run.map = map;
            run.map_progression = progression;
            run.boss_omen.mark_confirmed(node_id);
        }
        self.state.node_session = Some(enter_result.session.clone());

        let maybe_encounter_id = match &enter_result.payload {
            MapNodePayload::Encounter { encounter_id } => encounter_id.clone(),
            _ => None,
        };
        let entered_category = enter_result.category;
        let entered_payload = enter_result.payload.clone();
        if entered_category == MapNodeCategory::Gate {
            return self.handle_confirmed_gate_transition(node_id);
        }
        self.transition_to(GameState::InNode {
            node_id,
            kind_id: enter_result.kind_id.clone(),
            category: entered_category,
        })?;
        self.evaluate_visual_novel_triggers(
            super::visual_novel_triggers::VisualNovelTriggerEvent::MapNodeEntered {
                node_id,
                category: entered_category,
                kind_id: enter_result.kind_id.clone(),
            },
        )?;

        let entered = BehaviorResult::NodeEntered {
            node_id: enter_result.node_id,
            kind_id: enter_result.kind_id,
            category: enter_result.category,
            payload: enter_result.payload,
            session: enter_result.session,
        };

        if matches!(
            entered,
            BehaviorResult::NodeEntered {
                category: MapNodeCategory::Combat | MapNodeCategory::Boss,
                ..
            }
        ) {
            if let Some(encounter_id) = maybe_encounter_id {
                let primary_abnormality_id = self
                    .game_data
                    .pve_data
                    .get_by_id(&encounter_id)
                    .ok_or(GameError::MissingResource("PveEncounter"))?
                    .primary_abnormality_id
                    .clone();
                if let Some(primary_abnormality_id) = primary_abnormality_id.as_deref() {
                    let floor_index = self.run_state()?.run_progression.floor_index();
                    self.run_state_mut()?
                        .abnormality_encounter_history
                        .record_appearance(primary_abnormality_id, floor_index);
                }
                return self.handle_map_combat_node(
                    node_id,
                    primary_abnormality_id,
                    encounter_id,
                    battle_entry_snapshot.ok_or(GameError::InvalidAction)?,
                );
            }
        }

        if let Some(result) =
            self.try_enter_map_content_node(node_id, entered_category, &entered_payload)?
        {
            return Ok(result);
        }

        Ok(entered)
    }

    fn handle_confirmed_gate_transition(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<BehaviorResult, GameError> {
        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        if progression.current_node_id != Some(node_id) {
            return Err(GameError::InvalidAction);
        }
        let node = map.node(node_id).ok_or(GameError::InvalidAction)?;
        if node.category != MapNodeCategory::Gate {
            return Err(GameError::InvalidAction);
        }
        let template = self.facility_template_for_map(&map)?;
        progression
            .complete_current_node(&mut map, &template)
            .map_err(|_| GameError::InvalidAction)?;

        let mut run_progression = self.run_state()?.run_progression.clone();
        if !run_progression.advance_floor() {
            return Err(GameError::InvalidAction);
        }
        let abnormality_research = self.run_state()?.abnormality_research.clone();
        let abnormality_encounter_history = self.run_state()?.abnormality_encounter_history.clone();
        let monolith = self.run_state()?.monolith.clone();
        let mut boss_omen = self.run_state()?.boss_omen.clone();
        boss_omen.handle_gate_advance();
        let previous_facility_run_state = self.run_state()?.facility_run_state.clone();
        let (next_map, next_progression, facility_run_state) = self.generate_current_floor_map(
            &run_progression,
            &previous_facility_run_state,
            &abnormality_research,
            &abnormality_encounter_history,
            &monolith,
            &mut boss_omen,
        );
        let next_template = self.facility_template_for_map(&next_map)?;
        let view = next_progression.view(&next_map, &next_template);
        let (staged_skill_fragments, research_deliveries) =
            self.stage_current_planning_return_research_delivery()?;
        self.advance_employee_psychology_node()?;
        self.apply_staged_recovery_effect(StagedRecoveryEffect::GateTransition)?;
        let mut next_run = {
            let previous_run = self.run_state()?;
            RunState::for_next_floor_from(
                previous_run,
                next_map,
                next_progression,
                run_progression.clone(),
                boss_omen,
            )
            .with_facility_run_state(facility_run_state)
        };
        next_run.monolith.consume_next_floor_shop_guarantee();
        self.state.run = Some(next_run);
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;
        self.state.skill_fragments = staged_skill_fragments;
        Self::log_planning_return_research_deliveries(&research_deliveries);

        Ok(BehaviorResult::FloorAdvanced {
            game_mode: run_progression.game_mode,
            floor_index: run_progression.floor_index(),
            map: view,
            research_deliveries,
        })
    }

    pub(super) fn handle_complete_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_complete_node_inner)
    }

    fn handle_complete_node_inner(&mut self) -> Result<BehaviorResult, GameError> {
        let completion = self.plan_complete_current_node(true)?;
        self.commit_interactive_node_completion(completion)
    }

    pub(super) fn with_state_transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        let before = self
            .state
            .clone_non_battle_transaction_state()
            .ok_or(GameError::InvalidAction)?;
        match operation(self) {
            Ok(result) => Ok(result),
            Err(error) => {
                let encounter_compendium = self.state.encounter_compendium.clone();
                self.state = before;
                // Profile knowledge is a separate monotonic aggregate. Preserve
                // both the pre-transaction union and discoveries or defeats that
                // committed while the failed run-local operation was in flight.
                self.state.encounter_compendium.merge(encounter_compendium);
                Err(error)
            }
        }
    }

    pub(super) fn plan_complete_current_node(
        &mut self,
        apply_recovery_effect: bool,
    ) -> Result<StagedNodeCompletion, GameError> {
        if self
            .state
            .active_node_content
            .as_ref()
            .is_some_and(|content| matches!(content, ActiveNodeContent::HeadquartersContact(_)))
        {
            return Err(GameError::InvalidAction);
        }

        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let run_progression = self.run_state()?.run_progression.clone();
        let completing_node_id = progression.current_node_id;

        let recovery_effect = if apply_recovery_effect {
            self.plan_current_recovery_effect(&map, &progression)?
        } else {
            StagedRecoveryEffect::None
        };

        if !apply_recovery_effect {
            let mut boss_omen = self.run_state()?.boss_omen.clone();
            let template = self.facility_template_for_map(&map)?;
            if complete_forced_boss_with_exit_gate(
                &mut map,
                &mut progression,
                &template,
                &mut boss_omen,
            )
            .map_err(|_| GameError::InvalidAction)?
            {
                let view = progression.view(&map, &template);
                return Ok(StagedNodeCompletion::NodeCompleted {
                    map,
                    progression,
                    completing_node_id,
                    boss_omen: Some(boss_omen),
                    recovery_effect,
                    view,
                });
            }
        }

        let template = self.facility_template_for_map(&map)?;
        let terminal_resolution = progression
            .complete_current_node(&mut map, &template)
            .map_err(|_| GameError::InvalidAction)?;

        if terminal_resolution.is_none() {
            let manifested_rule_ids = map
                .manifest_satisfied_rules(&template)
                .map_err(GameError::InvalidStaticData)?;
            if !manifested_rule_ids.is_empty() {
                progression.refresh_available_nodes(&mut map, &template);
            }
            let view = progression.view(&map, &template);
            return Ok(StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen: None,
                recovery_effect,
                view,
            });
        }

        if !matches!(
            terminal_resolution,
            Some(
                FacilityTerminalResolution::FloorGate
                    | FacilityTerminalResolution::RunFinalBoss
                    | FacilityTerminalResolution::SpecialRunCompletion
            )
        ) {
            return Err(GameError::InvalidAction);
        }

        let mut run_progression = run_progression;

        if run_progression.advance_floor() {
            let abnormality_research = self.run_state()?.abnormality_research.clone();
            let abnormality_encounter_history =
                self.run_state()?.abnormality_encounter_history.clone();
            let monolith = self.run_state()?.monolith.clone();
            let mut boss_omen = self.run_state()?.boss_omen.clone();
            if let Some(node_id) = completing_node_id {
                boss_omen.consume_completed_source(node_id);
                boss_omen.clear_after_forced_boss_victory(node_id);
            }
            let previous_facility_run_state = self.run_state()?.facility_run_state.clone();
            let (next_map, next_progression, facility_run_state) = self.generate_current_floor_map(
                &run_progression,
                &previous_facility_run_state,
                &abnormality_research,
                &abnormality_encounter_history,
                &monolith,
                &mut boss_omen,
            );
            let next_template = self.facility_template_for_map(&next_map)?;
            let view = next_progression.view(&next_map, &next_template);
            Ok(StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                boss_omen,
                recovery_effect,
                view,
            })
        } else {
            let view = progression.view(&map, &template);
            Ok(StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                recovery_effect,
                view,
            })
        }
    }

    pub(super) fn commit_interactive_node_completion(
        &mut self,
        completion: StagedNodeCompletion,
    ) -> Result<BehaviorResult, GameError> {
        match completion {
            StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen,
                recovery_effect,
                view,
            } => {
                let (staged_skill_fragments, research_deliveries) =
                    self.stage_current_planning_return_research_delivery()?;
                self.commit_node_completed_with_recovery(
                    map,
                    progression,
                    completing_node_id,
                    boss_omen,
                    view,
                    recovery_effect,
                    Some(staged_skill_fragments),
                    research_deliveries,
                )
            }
            StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                boss_omen,
                recovery_effect,
                view,
            } => {
                let (staged_skill_fragments, research_deliveries) =
                    self.stage_current_planning_return_research_delivery()?;
                self.commit_floor_advanced_with_recovery(
                    next_map,
                    next_progression,
                    run_progression,
                    facility_run_state,
                    boss_omen,
                    view,
                    recovery_effect,
                    Some(staged_skill_fragments),
                    research_deliveries,
                )
            }
            StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                recovery_effect,
                view,
            } => self.commit_run_complete_with_recovery(
                map,
                progression,
                run_progression,
                view,
                recovery_effect,
            ),
        }
    }

    pub(super) fn commit_combat_result_node_completion(
        &mut self,
        completion: CombatResultNodeCompletion,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    ) -> Result<BehaviorResult, GameError> {
        match completion {
            CombatResultNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen,
                view,
            } => self.commit_node_completed_with_recovery(
                map,
                progression,
                completing_node_id,
                boss_omen,
                view,
                StagedRecoveryEffect::None,
                None,
                research_deliveries,
            ),
            CombatResultNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                boss_omen,
                view,
            } => self.commit_floor_advanced_with_recovery(
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                boss_omen,
                view,
                StagedRecoveryEffect::None,
                None,
                research_deliveries,
            ),
            CombatResultNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                view,
            } => self.commit_run_complete_without_recovery(map, progression, run_progression, view),
        }
    }

    fn commit_node_completed_with_recovery(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        planned_boss_omen: Option<BossOmenRunState>,
        mut view: MapViewDto,
        recovery_effect: StagedRecoveryEffect,
        staged_skill_fragments: Option<SkillFragmentInventory>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    ) -> Result<BehaviorResult, GameError> {
        let game_data = self.game_data.clone();
        if let Some(run) = self.state.run.as_mut() {
            let has_planned_boss_omen = planned_boss_omen.is_some();
            let mut boss_omen = planned_boss_omen.unwrap_or_else(|| run.boss_omen.clone());
            run.map = map;
            run.map_progression = progression;
            if let Some(node_id) = completing_node_id {
                if !has_planned_boss_omen {
                    boss_omen.consume_completed_source(node_id);
                    force_boss_node_if_ready(
                        &mut run.map,
                        &mut run.map_progression,
                        &run.run_progression,
                        game_data.as_ref(),
                        &mut boss_omen,
                    );
                }
                let template = game_data
                    .map_catalogs
                    .facility_templates
                    .template(&run.map.map_template_id)
                    .expect("committed run map template must remain live");
                view = run.map_progression.view(&run.map, template);
                run.clear_abnormality_attempt(node_id);
                run.event_sessions.remove(&node_id);
            }
            run.boss_omen = boss_omen;
        }
        self.advance_employee_psychology_node()?;
        self.apply_staged_recovery_effect(recovery_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;
        let completion = BehaviorResult::NodeCompleted {
            map: view,
            outcome: None,
            research_deliveries: Vec::new(),
        };
        let completion = self.finalize_combat_recovery_node_resolution(completion)?;
        let BehaviorResult::NodeCompleted { map, outcome, .. } = completion else {
            return Ok(completion);
        };
        if let Some(staged_skill_fragments) = staged_skill_fragments {
            self.state.skill_fragments = staged_skill_fragments;
        }
        Self::log_planning_return_research_deliveries(&research_deliveries);
        self.save_completion_checkpoint_if_needed(recovery_effect)?;
        Ok(BehaviorResult::NodeCompleted {
            map,
            outcome,
            research_deliveries,
        })
    }

    fn commit_floor_advanced_with_recovery(
        &mut self,
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        facility_run_state: FacilityRunState,
        boss_omen: BossOmenRunState,
        view: MapViewDto,
        recovery_effect: StagedRecoveryEffect,
        staged_skill_fragments: Option<SkillFragmentInventory>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    ) -> Result<BehaviorResult, GameError> {
        let next_run = {
            let previous_run = self.run_state()?;
            RunState::for_next_floor_from(
                previous_run,
                next_map,
                next_progression,
                run_progression.clone(),
                boss_omen,
            )
            .with_facility_run_state(facility_run_state)
        };
        self.state.run = Some(next_run);
        self.advance_employee_psychology_node()?;
        self.apply_staged_recovery_effect(recovery_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;
        if let Some(staged_skill_fragments) = staged_skill_fragments {
            self.state.skill_fragments = staged_skill_fragments;
        }
        Self::log_planning_return_research_deliveries(&research_deliveries);
        self.save_completion_checkpoint_if_needed(recovery_effect)?;
        Ok(BehaviorResult::FloorAdvanced {
            game_mode: run_progression.game_mode,
            floor_index: run_progression.floor_index(),
            map: view,
            research_deliveries,
        })
    }

    fn commit_run_complete_without_recovery(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
    ) -> Result<BehaviorResult, GameError> {
        self.commit_run_complete_with_recovery(
            map,
            progression,
            run_progression,
            view,
            StagedRecoveryEffect::None,
        )
    }

    fn commit_run_complete_with_recovery(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
        recovery_effect: StagedRecoveryEffect,
    ) -> Result<BehaviorResult, GameError> {
        let completed_run = {
            let previous_run = self.run_state()?;
            RunState::for_run_complete_from(previous_run, map, progression, run_progression)
        };
        self.state.run = Some(completed_run);
        self.advance_employee_psychology_node()?;
        self.apply_staged_recovery_effect(recovery_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::RunComplete)?;
        self.save_completion_checkpoint_if_needed(recovery_effect)?;
        Ok(BehaviorResult::RunComplete { map: view })
    }

    fn save_completion_checkpoint_if_needed(
        &mut self,
        recovery_effect: StagedRecoveryEffect,
    ) -> Result<(), GameError> {
        if recovery_effect == StagedRecoveryEffect::SavePoint {
            self.save_run_checkpoint()?;
        }
        Ok(())
    }

    fn advance_employee_psychology_node(&mut self) -> Result<(), GameError> {
        for employee in self.roster_mut()?.iter_mut() {
            employee.psychology.advance_node();
        }
        Ok(())
    }

    pub(super) fn save_run_checkpoint(&mut self) -> Result<(), GameError> {
        let run = self.run_state()?;
        self.state.run_checkpoint.payload = Some(RunCheckpointPayload {
            power_model_version: self.state.run_power_authority.power_model_version,
            current_floor_power_anchor: self.state.run_power_authority.current_floor_power_anchor,
            map: run.map.clone(),
            map_progression: run.map_progression.clone(),
            run_progression: run.run_progression.clone(),
            facility_run_state: run.facility_run_state.clone(),
            abnormality_research: run.abnormality_research.clone(),
            abnormality_encounter_history: run.abnormality_encounter_history.clone(),
            monolith: run.monolith.clone(),
            boss_omen: run.boss_omen.clone(),
            event_sessions: run.event_sessions.clone(),
            shop_sessions: run.shop_sessions.clone(),
            combat_previews: run.combat_previews.clone(),
            abnormality_attempts: run.abnormality_attempts.clone(),
            recruitment: run.recruitment.clone(),
            visual_novel_trigger_checkpoint_consumed: self
                .state
                .visual_novel_triggers
                .checkpoint_rewind_consumed
                .clone(),
            enkephalin: self.state.enkephalin.clone(),
            inventory: self.state.inventory.clone(),
            roster: self.state.roster.clone(),
            skill_fragments: self.state.skill_fragments.clone(),
            uuid_manager: self.state.uuid_manager.clone(),
        });
        Ok(())
    }

    pub(super) fn handle_load_run_checkpoint(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_load_run_checkpoint_inner)
    }

    fn handle_load_run_checkpoint_inner(&mut self) -> Result<BehaviorResult, GameError> {
        if !matches!(
            self.state.game_state,
            GameState::ViewingMap
                | GameState::NodeConfirm { .. }
                | GameState::AwaitingRunFailureDecision { .. }
        ) {
            return Err(GameError::InvalidAction);
        }
        let payload = self
            .state
            .run_checkpoint
            .payload
            .clone()
            .ok_or(GameError::InvalidAction)?;
        self.validate_run_checkpoint_payload(&payload)?;
        if self.state.run_checkpoint.loads_used >= RUN_CHECKPOINT_MAX_LOADS {
            return Err(GameError::InvalidAction);
        }
        let next_loads_used = self.state.run_checkpoint.loads_used.saturating_add(1);
        self.restore_run_checkpoint_payload(payload)?;
        self.state.run_checkpoint.loads_used = next_loads_used;
        self.handle_request_map_data()
    }

    pub(super) fn restore_run_checkpoint_payload(
        &mut self,
        payload: RunCheckpointPayload,
    ) -> Result<(), GameError> {
        self.state.run_power_authority.power_model_version = payload.power_model_version;
        self.state.run_power_authority.current_floor_power_anchor =
            payload.current_floor_power_anchor;
        self.state.run = Some(RunState {
            map: payload.map,
            map_progression: payload.map_progression,
            run_progression: payload.run_progression,
            facility_run_state: payload.facility_run_state,
            abnormality_research: payload.abnormality_research,
            abnormality_encounter_history: payload.abnormality_encounter_history,
            monolith: payload.monolith,
            boss_omen: payload.boss_omen,
            event_sessions: payload.event_sessions,
            shop_sessions: payload.shop_sessions,
            combat_previews: payload.combat_previews,
            abnormality_attempts: payload.abnormality_attempts,
            recruitment: payload.recruitment,
            combat_recovery_restriction: None,
        });
        self.state.enkephalin = payload.enkephalin;
        self.state.inventory = payload.inventory;
        self.state.roster = payload.roster;
        self.state.skill_fragments = payload.skill_fragments;
        self.state
            .uuid_manager
            .restore_checkpoint_preserving_counters(
                payload.uuid_manager,
                &[crate::game::managers::uuid_manager::UuidManager::NS_VISUAL_NOVEL_SESSION],
            );
        self.state.visual_novel_triggers.checkpoint_rewind_consumed =
            payload.visual_novel_trigger_checkpoint_consumed;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.active_battle = None;
        self.transition_to(GameState::ViewingMap)?;
        Ok(())
    }

    pub(super) fn checkpoint_payload_is_loadable(&self) -> bool {
        self.state.run_checkpoint.loads_used < RUN_CHECKPOINT_MAX_LOADS
            && self
                .state
                .run_checkpoint
                .payload
                .as_ref()
                .is_some_and(|payload| self.validate_run_checkpoint_payload(payload).is_ok())
    }

    pub(super) fn validate_run_checkpoint_payload(
        &self,
        payload: &RunCheckpointPayload,
    ) -> Result<(), GameError> {
        if payload.power_model_version != self.run_policy().combat_difficulty.power_model_version
            || payload.current_floor_power_anchor == 0
        {
            return Err(GameError::InvalidStaticData(
                "run checkpoint power authority does not match live policy".to_string(),
            ));
        }
        if payload.map.nodes.iter().any(|node| {
            let Some(difficulty) = node.combat_difficulty.as_ref() else {
                return validate_monolith_difficulty_authority(node.id, None, &payload.monolith)
                    .is_err();
            };
            let Some(tuning) = self
                .run_policy()
                .resolved_tuning_for_manifested_node(difficulty)
            else {
                return true;
            };
            difficulty.power_model_version != payload.power_model_version
                || difficulty.floor_power_anchor != payload.current_floor_power_anchor
                || difficulty.target_power
                    != percent_of_anchor_ceil(
                        difficulty.floor_power_anchor,
                        tuning.target_power_percent,
                    )
                || validate_monolith_difficulty_authority(
                    node.id,
                    Some(difficulty),
                    &payload.monolith,
                )
                .is_err()
        }) {
            return Err(GameError::InvalidStaticData(
                "run checkpoint map difficulty does not match its power authority".to_string(),
            ));
        }
        let template = self.facility_template_for_map(&payload.map)?;
        payload
            .map
            .validate_facility_template_contract(&template)
            .map_err(GameError::InvalidStaticData)?;
        let references_existing_node = |node_id: &MapNodeId| payload.map.node(*node_id).is_some();
        if payload
            .map_progression
            .current_node_id
            .as_ref()
            .is_some_and(|node_id| !references_existing_node(node_id))
            || payload
                .map_progression
                .available_node_ids
                .iter()
                .any(|node_id| !references_existing_node(node_id))
            || payload
                .map_progression
                .completed_node_ids
                .iter()
                .any(|node_id| !references_existing_node(node_id))
        {
            return Err(GameError::InvalidStaticData(
                "run checkpoint progression references a missing map node".to_string(),
            ));
        }
        payload.recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            &payload.roster,
        )?;
        payload
            .monolith
            .validate_checkpoint_state(
                self.run_policy().monolith.initial_uses,
                self.run_policy().monolith.residual_exchange_threshold,
                |node_id| payload.map.node(node_id).is_some(),
                |abnormality_id| {
                    self.game_data.pve_data.encounters.iter().any(|encounter| {
                        encounter.encounter_class
                            == crate::game::data::pve_data::PveEncounterClass::Elite
                            && encounter.primary_abnormality_id() == Some(abnormality_id)
                    })
                },
                |encounter_id, abnormality_id| {
                    self.game_data
                        .pve_data
                        .get_by_id(encounter_id)
                        .is_some_and(|encounter| {
                            encounter.encounter_class
                                == crate::game::data::pve_data::PveEncounterClass::Elite
                                && encounter.primary_abnormality_id() == Some(abnormality_id)
                        })
                },
            )
            .map_err(GameError::InvalidStaticData)?;
        Ok(())
    }

    pub(super) fn validate_live_monolith_difficulty_authority(
        &self,
        node_id: MapNodeId,
        difficulty: Option<&crate::game::map::MapNodeCombatDifficulty>,
    ) -> Result<(), GameError> {
        validate_monolith_difficulty_authority(node_id, difficulty, &self.run_state()?.monolith)
    }
}

#[cfg(test)]
mod monolith_difficulty_authority_tests {
    use super::*;
    use crate::game::monolith::{MonolithRemanifestationNode, MonolithRunState};

    fn difficulty(
        stage: Option<MonolithRemanifestationStage>,
    ) -> crate::game::map::MapNodeCombatDifficulty {
        crate::game::map::MapNodeCombatDifficulty {
            power_model_version: 2,
            difficulty_profile_id: "default_facility".to_string(),
            class: MapNodeCombatDifficultyClass::Role(
                crate::game::data::run_policy_data::CombatNodeDifficultyRole::MonolithRemanifestation,
            ),
            monolith_stage: stage,
            floor_power_anchor: 100,
            target_power: 220,
        }
    }

    #[test]
    fn typed_monolith_stage_must_match_ledger_stage() {
        let node_id = MapNodeId::new(uuid::Uuid::from_u128(7));
        let mut monolith = MonolithRunState::default();
        monolith.remanifestation_nodes.insert(
            node_id,
            MonolithRemanifestationNode {
                abnormality_id: "abno".to_string(),
                encounter_id: "elite_abno".to_string(),
                stage: 2,
            },
        );

        assert!(validate_monolith_difficulty_authority(
            node_id,
            Some(&difficulty(Some(MonolithRemanifestationStage::Stage2))),
            &monolith,
        )
        .is_ok());
        assert!(validate_monolith_difficulty_authority(
            node_id,
            Some(&difficulty(Some(MonolithRemanifestationStage::Stage3))),
            &monolith,
        )
        .is_err());
        assert!(validate_monolith_difficulty_authority(node_id, None, &monolith).is_err());
    }
}
