use std::collections::HashMap;

use tracing::info;
#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
use tracing::warn;
use uuid::Uuid;

use super::{
    expedition_recovery::RecoveryContext,
    grant_state::EconomyGrantState,
    node_flow::CombatResultNodeCompletion,
    state::{
        ActiveBattleSession, BattleEmployeeAttemptState, BattleEntrySnapshot,
        CombatRecoveryRestriction, LiveBattleDeployedUnitState, LiveBattleDeploymentState,
        LiveBattleRedeployState, TacticalResourceState,
    },
    GameCore,
};
use crate::game::ability::{SkillCastSelection, SkillId};
use crate::game::battle::{
    core::sim::{
        BattleDeployCurrentHpPolicy, BattleLiveCommand, BattleLiveCommandOutcome, BattleStepOutcome,
    },
    result_stats::{BattleResultEmployeeStatsDto, BattleResultStatsDto},
    runtime_control::invariant,
    tile_range::FacingDirection,
    types::{BattleUnitSource, BattleWinner, ParticipantBattleResult},
};
use crate::game::behavior::{
    BattlePayloadEligibilityDto, BattlePlaybackState, BehaviorResult, CombatOutcomeSummary,
    DeploymentRangePreviewFacingDto, DeploymentRangePreviewFacingsDto,
    DeploymentRangePreviewResultDto, GameError, LiveBattleRangePreviewsDto,
    LiveBattleTacticalFocusPurposeDto, NodeOutcomeEmployeeChange, NodeOutcomeSummary,
    SkillTargetPreviewResultDto,
};
use crate::game::combat_preview::{CombatPreview, DeploymentZoneKind};
use crate::game::combat_setup::player_spawns::{
    battle_unit_draft_for_employee, effective_combat_profile_for_employee,
};
use crate::game::combat_setup::rewards::resolve_combat_outcome_policies;
use crate::game::data::run_policy_data::PostBattleResolutionPolicy;
use crate::game::employee::EmployeeRoster;
use crate::game::encounter_compendium::EncounterCompendiumProfileState;
use crate::game::encounter_progression::EncounterProgressionState;
use crate::game::enums::RewardMode;
use crate::game::events::combat::{CombatExecutor, ResolvedCombatSetupPlan};
use crate::game::map::{
    GameMode, MapNodeCategory, MapNodeExecutor, MapNodeId, MapNodePayload,
};
use crate::game::range_preview::range_previews_for_combat_profile;
use crate::game::resources::{
    ActiveNodeContent, CombatBattleState, EquipmentLoadoutTransition, GameState, Inventory,
    InventoryDiffDto, InventoryItemDto, Position, RunFailureReason,
};
use crate::game::reward::{
    EmployeeExperienceDiffDto, ExperienceTargetPolicy, GrantExecutionContext, RewardEffect,
    RewardOption, SkillFragmentGrantDiffDto,
};
use crate::game::skill_fragment::SkillFragmentInventory;

#[derive(Debug, Clone, Copy)]
struct EmployeeOutcomeBefore {
    physical_damage: u8,
    trauma: u32,
    experience: u32,
    alive: bool,
}

struct StagedCombatResultState {
    grant_state: EconomyGrantState,
    condition_inventory_diff: InventoryDiffDto,
    employee_died: bool,
}

pub(super) struct PlannedLiveBattleStart {
    combat_preview: CombatPreview,
    active_battle: ActiveBattleSession,
    entry_snapshot: Option<BattleEntrySnapshot>,
}

#[derive(Clone, Copy)]
enum BattleAdvanceStep {
    Elapsed(u64),
    ServerTick(u64),
    NextEventBucket,
}

#[derive(Clone, Copy)]
enum CombatPreviewResolutionMode<'a> {
    Presentation,
    BattleStart {
        category: MapNodeCategory,
        encounter_id: &'a str,
    },
}

enum CombatPreviewResolution {
    NotApplicable,
    CacheHit(CombatPreview),
    Generated(CombatPreview),
}

struct CombatPreviewResolutionInput {
    actual_category: MapNodeCategory,
    generation_category: MapNodeCategory,
    encounter_id: Option<String>,
    existing: Option<CombatPreview>,
    floor_index: u32,
    difficulty: Option<crate::game::map::MapNodeCombatDifficulty>,
}

impl ActiveBattleSession {
    fn reconcile_employee_attempt(
        &mut self,
        post_battle_policy: PostBattleResolutionPolicy,
    ) -> Result<(), GameError> {
        for unit in self.battle.units.values().filter(|unit| unit.corroded) {
            let crate::game::battle::types::BattleUnitSourceIdentity::Employee {
                employee_uuid,
                ..
            } = unit.source_identity
            else {
                return Err(invariant("reconcile_corroded_source"));
            };
            let employee = self
                .employee_attempt
                .roster
                .get_mut(&employee_uuid)
                .ok_or_else(|| invariant("reconcile_corroded_roster"))?;
            employee.condition.trauma = 100;
        }
        let incapacitations = self.unprocessed_deployed_employee_incapacitations();
        for (employee_uuid, _) in &incapacitations {
            if self.employee_attempt.roster.get(employee_uuid).is_none() {
                return Err(invariant("reconcile_incapacitated_roster"));
            }
        }

        for (employee_uuid, unit_instance_id) in incapacitations {
            let employee = self
                .employee_attempt
                .roster
                .get_mut(&employee_uuid)
                .expect("employee incapacitations were prevalidated");
            if self.battle.units[&unit_instance_id].lifecycle == crate::game::battle::core::types::RuntimeUnitLifecycle::PermanentlyDead {
                employee.condition.physical_damage = crate::game::employee::MAX_PHYSICAL_DAMAGE;
            } else {
                employee.apply_incapacitation(post_battle_policy.incapacitation_trauma);
            }
            self.employee_attempt
                .processed_incapacitations
                .insert(unit_instance_id);
            info!(
                "Employee {} tactically incapacitated in unit {}: physical_damage={}, trauma={}",
                employee.uuid,
                unit_instance_id,
                employee.condition.physical_damage,
                employee.condition.trauma
            );
        }

        self.reconcile_live_deployment_with_battle_state();
        self.reconcile_tactical_focus();
        Ok(())
    }
}

enum PlannedPlayerVictoryCombatResultCompletion {
    RunFailure {
        staged: StagedCombatResultState,
        reason: RunFailureReason,
        encounter_registrations: EncounterCompendiumProfileState,
    },
    RewardsGranted {
        staged: StagedCombatResultState,
        encounter_registrations: EncounterCompendiumProfileState,
        encounter_progression_after_victory: Option<EncounterProgressionState>,
        node_completion: CombatResultNodeCompletion,
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        employee_experience_diffs: Vec<EmployeeExperienceDiffDto>,
        outcome: NodeOutcomeSummary,
        recovery_restriction: Option<PlannedCombatRecoveryRestriction>,
        pending_reward: Option<crate::game::resources::RewardSessionState>,
        experience_context: GrantExecutionContext,
    },
}

struct PlannedCombatRecoveryRestriction {
    allowed_node_ids: Vec<MapNodeId>,
    path_exists: bool,
}

#[derive(Default)]
struct EncounterVictoryPlan {
    progression_after: Option<EncounterProgressionState>,
    reward_effects: Vec<RewardEffect>,
}

impl GameCore {
    pub(super) fn combat_preview_for_node(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<Option<CombatPreview>, GameError> {
        if self.current_facility_encounter_hidden(node_id) { return Ok(None); }
        match self
            .resolve_combat_preview_for_node(node_id, CombatPreviewResolutionMode::Presentation)?
        {
            CombatPreviewResolution::NotApplicable => Ok(None),
            CombatPreviewResolution::CacheHit(preview) => Ok(Some(preview)),
            CombatPreviewResolution::Generated(preview) => {
                self.run_state_mut()?
                    .combat_previews
                    .insert(node_id, preview.clone());
                Ok(Some(preview))
            }
        }
    }

    pub(super) fn deployment_zone_kind_for_cell(
        combat_preview: &CombatPreview,
        position: Position,
    ) -> Option<DeploymentZoneKind> {
        combat_preview
            .deployment_zones
            .iter()
            .find(|zone| zone.cells.contains(&position))
            .map(|zone| zone.kind)
    }

    fn validate_employee_deployment_cell(
        &self,
        combat_preview: &CombatPreview,
        employee_uuid: Uuid,
        position: Position,
    ) -> Result<(), GameError> {
        let zone_kind = Self::deployment_zone_kind_for_cell(combat_preview, position)
            .ok_or(GameError::OutOfBounds)?;
        let profile = effective_combat_profile_for_employee(
            self.roster()?,
            self.inventory()?,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
            employee_uuid,
        )?;
        if !zone_kind.supports_affinity(profile.deployment_affinity) {
            return Err(GameError::InvalidAction);
        }
        if let Some(active) = self.state.active_battle.as_ref() {
            active
                .battle
                .validate_player_tactical_destination(position, None)?;
        }
        Ok(())
    }

    pub(super) fn handle_map_combat_node(
        &mut self,
        node_id: MapNodeId,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        entry_snapshot: BattleEntrySnapshot,
    ) -> Result<BehaviorResult, GameError> {
        let category = self
            .run_state()?
            .map
            .node(node_id)
            .ok_or(GameError::InvalidAction)?
            .category;
        self.handle_explicit_combat_node(
            node_id,
            category,
            primary_abnormality_id,
            encounter_id,
            Some(entry_snapshot),
        )
    }

    pub(super) fn handle_event_combat_node(
        &mut self,
        node_id: MapNodeId,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
    ) -> Result<BehaviorResult, GameError> {
        let enter_result = {
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            MapNodeExecutor::enter(node)
        };
        let entry_snapshot = BattleEntrySnapshot::capture(&self.state)?.as_event_combat_reentry(
            node_id,
            enter_result.kind_id,
            enter_result.category,
            enter_result.session,
        );
        self.handle_explicit_combat_node(
            node_id,
            MapNodeCategory::Combat,
            primary_abnormality_id,
            encounter_id,
            Some(entry_snapshot),
        )
    }

    fn explicit_combat_preview_for_node(
        &self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        encounter_id: &str,
    ) -> Result<CombatPreview, GameError> {
        // Persisted previews remain presentation caches. Battle setup must derive a
        // fresh deterministic instance from the current run node and live policy,
        // because internal setup fields are intentionally omitted from serialization.
        match self.resolve_combat_preview_for_node(
            node_id,
            CombatPreviewResolutionMode::BattleStart {
                category,
                encounter_id,
            },
        )? {
            CombatPreviewResolution::Generated(preview) => Ok(preview),
            CombatPreviewResolution::NotApplicable | CombatPreviewResolution::CacheHit(_) => {
                unreachable!("battle-start preview resolution always generates fresh authority")
            }
        }
    }

    fn resolve_combat_preview_for_node(
        &self,
        node_id: MapNodeId,
        mode: CombatPreviewResolutionMode<'_>,
    ) -> Result<CombatPreviewResolution, GameError> {
        let input = {
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            let (generation_category, encounter_id, existing) = match mode {
                CombatPreviewResolutionMode::Presentation => {
                    let encounter_id = match &node.payload {
                        MapNodePayload::Encounter { encounter_id } => encounter_id.clone(),
                        _ => None,
                    };
                    (
                        node.category,
                        encounter_id,
                        run.combat_previews.get(&node_id).cloned(),
                    )
                }
                CombatPreviewResolutionMode::BattleStart {
                    category,
                    encounter_id,
                } => (category, Some(encounter_id.to_string()), None),
            };
            CombatPreviewResolutionInput {
                actual_category: node.category,
                generation_category,
                encounter_id,
                existing,
                floor_index: run.run_progression.floor_index(),
                difficulty: node.combat_difficulty.clone(),
            }
        };

        if matches!(mode, CombatPreviewResolutionMode::Presentation) {
            // Event nodes can start authored combat while retaining their Event category.
            // Retry must reuse the committed preview before the manifested category gate.
            if let Some(mut existing) = input.existing {
                self.apply_deterrence_preview(node_id, &mut existing)?;
                return Ok(CombatPreviewResolution::CacheHit(existing));
            }
            if !matches!(
                input.actual_category,
                MapNodeCategory::Combat | MapNodeCategory::Boss
            ) {
                return Ok(CombatPreviewResolution::NotApplicable);
            }
        }

        let difficulty = input.difficulty.as_ref();
        if difficulty.is_none()
            && matches!(
                input.actual_category,
                MapNodeCategory::Combat | MapNodeCategory::Boss
            )
        {
            return Err(GameError::InvalidStaticData(format!(
                "manifested {:?} node '{}' is missing combat difficulty authority",
                input.actual_category, node_id.0
            )));
        }
        if matches!(mode, CombatPreviewResolutionMode::Presentation) {
            self.validate_live_revival_difficulty_authority(node_id, difficulty)?;
        }

        let seed = self.node_seed(node_id, 0x5052_4556); // "PREV"
        let mut preview = CombatPreview::try_generate_for_node_at_floor_with_difficulty(
            node_id,
            input.generation_category,
            input.encounter_id.as_deref(),
            self.game_data.as_ref(),
            seed,
            input.floor_index,
            difficulty,
        )?;
        self.apply_deterrence_preview(node_id, &mut preview)?;
        Ok(CombatPreviewResolution::Generated(preview))
    }

    fn apply_deterrence_preview(&self, node_id: MapNodeId, preview: &mut CombatPreview) -> Result<(), GameError> {
        let run = self.run_state()?;
        let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
        let unrestrained = self.node_entry_preview(node)?.unrestrained_after_entry;
        let encounter_id = match &node.payload {
            MapNodePayload::Encounter { encounter_id: Some(id) } => Some(id.as_str()),
            _ => preview.encounter_id.as_deref(),
        };
        let encounter = encounter_id.and_then(|id| self.game_data.pve_data.get_by_id(id))
            .ok_or(GameError::InvalidAction)?;
        let tuning = encounter.deterrence_tuning;
        let sealed = self.sealed_golden_bough(node_id).is_some();
        let snapshot = crate::game::combat_preview::BattleDeterrenceSnapshot::for_entry(tuning, unrestrained, sealed);
        preview.enemy_stat_scale.encounter_health_percent = snapshot.enemy_health_percent;
        preview.enemy_stat_scale.encounter_attack_percent = snapshot.enemy_attack_percent;
        preview.deterrence = snapshot;
        Ok(())
    }

    fn handle_explicit_combat_node(
        &mut self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        entry_snapshot: Option<BattleEntrySnapshot>,
    ) -> Result<BehaviorResult, GameError> {
        if let Some(result) = self.block_or_fail_undeployable_combat_selection()? {
            return Ok(result);
        }
        let planned = self.plan_explicit_combat_node_with_state(
            node_id,
            category,
            primary_abnormality_id,
            encounter_id,
            &self.state.inventory,
            &self.state.skill_fragments,
            &self.state.roster,
        )?;
        let mut planned = planned;
        planned.entry_snapshot = entry_snapshot;
        self.commit_planned_live_battle_start(node_id, planned)
    }

    pub(super) fn plan_event_combat_node_with_state(
        &self,
        node_id: MapNodeId,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        roster: &EmployeeRoster,
    ) -> Result<PlannedLiveBattleStart, GameError> {
        self.plan_explicit_combat_node_with_state(
            node_id,
            MapNodeCategory::Combat,
            primary_abnormality_id,
            encounter_id,
            inventory,
            skill_fragments,
            roster,
        )
    }

    pub(super) fn plan_explicit_combat_node_with_state(
        &self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        inventory: &Inventory,
        skill_fragments: &SkillFragmentInventory,
        roster: &EmployeeRoster,
    ) -> Result<PlannedLiveBattleStart, GameError> {
        info!(
            "Starting map combat node for primary_abnormality={:?} encounter={} node={}",
            primary_abnormality_id, encounter_id, node_id.0
        );

        let combat_preview =
            self.explicit_combat_preview_for_node(node_id, category, &encounter_id)?;
        let node_type = combat_preview.node_type;
        let mission_variant = combat_preview.mission_variant;
        let reward_multiplier_percent = self.combat_reward_multiplier_for_node(node_id)?;
        let authored_policy = self.authored_combat_policy_for_node(node_id)?;

        if crate::game::combat_setup::mission_policy::CombatMissionPolicy::starts_as_live_battle(
            node_type,
            mission_variant,
        ) {
            if self
                .run_state()?
                .abnormality_attempt_state(node_id)
                .remaining_attempts()
                == 0
            {
                return Err(GameError::InvalidAction);
            }
            let empty_deployment = HashMap::new();
            let ResolvedCombatSetupPlan {
                node_type,
                mission_variant,
                outcome_policies,
                mut battle,
                execution,
            } = CombatExecutor::resolve_live_combat_setup_plan_with_authored_policy(
                roster,
                inventory,
                skill_fragments,
                self.game_data.database_arc(),
                Some(self.game_data.identity_arc()),
                primary_abnormality_id.as_deref(),
                &encounter_id,
                self.run_seed,
                &combat_preview,
                &empty_deployment,
                reward_multiplier_percent,
                authored_policy.as_ref(),
                self.authoring_observation
                    .as_ref()
                    .map(|owner| (owner.trace_options, owner.amount_observation_enabled)),
                self.state
                    .has_relic_owner()
                    .then_some(crate::game::events::combat::RelicBattleRuntimeOwner),
                crate::game::battle::core::relic_numeric::RelicNumericOwner::capture(
                    &self.state.relics,
                    self.game_data.database_arc(),
                    self.game_data.identity_arc(),
                )?,
            )?;
            let stress_combat = if self.basic_relic_activity(node_id) == Some(crate::game::map::RewardActivityKind::StressCombat) {
                let policy = self.facility_layout().and_then(|l| l.stress_combat.as_ref()).ok_or(GameError::InvalidAction)?;
                crate::game::relic::allocation::admit_owned_projection(policy, "stress_battle_policy")?;
                let policy = crate::authoring::clone_shape::controlled_clone(policy)?;
                battle.configure_stress_rounds(policy.battle_round_groups()?);
                Some(policy)
            } else { None };
            battle.configure_performance_bonus(primary_abnormality_id.as_deref());
            battle.set_effective_owner(self.capture_structural_battle_owner()?)?;
            for employee in roster.iter() {
                for equipped in employee.loadout.item_slot.iter() {
                    let owned = inventory.equipments.get_item(&equipped.instance_uuid)
                        .ok_or(GameError::InventoryItemNotFound)?;
                    if owned.meta.uuid != equipped.base_uuid { return Err(GameError::InvalidAction); }
                    battle.register_equipment_cohort(employee.uuid, equipped.instance_uuid, equipped.base_uuid, 0)?;
                }
            }
            for employee in roster.iter(){
                let Some(copy_uuid)=employee.skill_fragments.active_copy_uuid() else {continue;};
                let copy=skill_fragments.get(&copy_uuid).ok_or(GameError::InvalidAction)?;
                if copy.equipped_to!=Some(employee.uuid){return Err(GameError::InvalidAction);}
                let metadata=self.game_data.skill_fragment_data.get_by_id(&copy.definition_id).ok_or(GameError::InvalidAction)?;
                let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill{forms,..}=&metadata.effect else {return Err(GameError::InvalidAction);};
                let form=forms.get(&copy.selected_form).ok_or(GameError::InvalidAction)?;
                let fragment=crate::game::skill_fragment::FragmentSourceSnapshot{copy_uuid,definition_id:copy.definition_id.clone(),form_id:copy.selected_form.clone(),skill_id:form.skill_id.clone()};
                let fragment=fragment.into_shared()?;
                battle.register_input_plan_copy(employee.uuid,fragment.clone(),0)?;
                battle.register_prepared_volley_copy(employee.uuid,fragment,self.state.run.as_ref().and_then(|r|r.credit_ledgers.entries.get(&copy_uuid)))?;
            }
            if let Some(run)=&self.state.run{battle.mount_credit_ledgers(&run.credit_ledgers)?;}
            if let Some(plan)=self.authoring_observation.as_ref().and_then(|o|o.coverage_plan.as_ref()) {battle.set_lab_coverage(plan)?;}
            let live_deployment_policy = self.run_policy().live_deployment;
            let tactical_resource_policy = self.run_policy().tactical_resource;
            let active_battle = ActiveBattleSession {
                node_id,
                battle_uuid: Uuid::nil(),
                battle_state_revision: 0,
                primary_abnormality_id,
                encounter_id,
                node_type,
                mission_variant,
                outcome_policies,
                stress_combat,
                combat_preview: combat_preview.clone(),
                battle,
                execution,
                entry_snapshot: None,
                employee_attempt: BattleEmployeeAttemptState::new(roster)?,
                tactical_resource: TacticalResourceState::new(tactical_resource_policy),
                live_deployment: Some(LiveBattleDeploymentState::new(live_deployment_policy)),
                playback: BattlePlaybackState::default(),
                playback_delta_remainder: 0,
                tactical_focus: None,
                interaction: Default::default(),
            };
            return Ok(PlannedLiveBattleStart {
                combat_preview,
                active_battle,
                entry_snapshot: None,
            });
        }

        Err(GameError::InvalidStaticData(format!(
            "combat encounter '{}' uses unsupported non-live combat mission {:?}/{:?}; official combat flow is DefenseRoute live battle only",
            encounter_id, node_type, mission_variant
        )))
    }

    fn combat_reward_multiplier_for_node(&self, node_id: MapNodeId) -> Result<u32, GameError> {
        let node = self
            .run_state()?
            .map
            .node(node_id)
            .ok_or(GameError::InvalidAction)?;
        let Some(difficulty) = node.combat_difficulty.as_ref() else {
            if matches!(
                node.category,
                MapNodeCategory::Combat | MapNodeCategory::Boss
            ) {
                return Err(GameError::InvalidStaticData(format!(
                    "manifested {:?} node '{}' is missing combat difficulty authority",
                    node.category, node.id.0
                )));
            }
            // Event-authored combat is not a manifested Combat/Boss node and keeps its
            // authored reward contract unchanged.
            return Ok(100);
        };
        self.validate_live_revival_difficulty_authority(node_id, Some(difficulty))?;
        self.run_policy()
            .resolved_tuning_for_manifested_node(difficulty)
            .map(|tuning| tuning.reward_multiplier_percent)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "manifested combat difficulty '{}@{}' does not match live reward policy",
                    difficulty.difficulty_profile_id, difficulty.power_model_version
                ))
            })
    }

    fn authored_combat_policy_for_node(
        &self,
        node_id: MapNodeId,
    ) -> Result<Option<crate::game::map::AuthoredCombatNodePolicy>, GameError> {
        let run = self.run_state()?;
        if run.run_progression.game_mode != GameMode::Tutorial {
            return Ok(None);
        }
        let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
        let schedule = self
            .game_data
            .map_catalogs
            .node_assignment_profiles
            .resolve(
                &self.game_data.map_catalogs.facility_templates,
                GameMode::Tutorial,
                run.run_progression.floor_index(),
            )
            .map_err(GameError::InvalidStaticData)?;
        Ok(schedule
            .profile
            .fixed_slot_assignments
            .iter()
            .find(|assignment| {
                assignment.slot_id == node.slot_id
                    && assignment.kind_id == node.kind_id
                    && assignment.payload == node.payload
            })
            .and_then(|assignment| assignment.authored_combat.clone()))
    }

    pub(super) fn commit_planned_live_battle_start(
        &mut self,
        node_id: MapNodeId,
        mut planned: PlannedLiveBattleStart,
    ) -> Result<BehaviorResult, GameError> {
        self.reconcile_run_resources()?;
        let owners = super::resource_runtime::resource_owners(
            &self.game_data.resource_data,
            &self.state.roster,
            &self.state.inventory,
            &self.state.skill_fragments,
        )?;
        planned
            .active_battle
            .battle
            .mount_run_resources(&self.run_state()?.typed_resources, &owners)?;
        let certified = crate::game::combat_power::certified_combat_power_for_roster(
            &self.state.roster,
            &self.state.inventory,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
        )?;
        let entry_snapshot = if let Some(snapshot) = planned.entry_snapshot.take() {
            snapshot
        } else if self
            .state
            .node_session
            .as_ref()
            .is_some_and(|session| session.category == MapNodeCategory::Event)
        {
            let enter_result = {
                let run = self.run_state()?;
                let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
                MapNodeExecutor::enter(node)
            };
            BattleEntrySnapshot::capture(&self.state)?.as_event_combat_reentry(
                node_id,
                enter_result.kind_id,
                enter_result.category,
                enter_result.session,
            )
        } else {
            BattleEntrySnapshot::capture(&self.state)?
        };
        let entry_snapshot = entry_snapshot.freeze()?;
        if self
            .run_state_mut()?
            .start_abnormality_attempt(node_id)
            .is_none()
        {
            return Err(GameError::InvalidAction);
        }
        self.state
            .run_power_authority
            .record_combat_entry(certified.power_model_version, certified.total);
        self.run_state_mut()?
            .combat_previews
            .insert(node_id, planned.combat_preview);

        planned.active_battle.battle_uuid = self.state.uuid_manager.next_battle_session();
        planned.active_battle.entry_snapshot = Some(entry_snapshot);
        self.record_lab_definition("PveEncounter",&planned.active_battle.encounter_id,"encounter_plan")?;
        let battle_uuid = planned.active_battle.battle_uuid;
        self.state.active_battle = Some(planned.active_battle);
        self.transition_to(GameState::InBattle { battle_uuid })?;
        self.authoring_battle_entered(battle_uuid)?;
        let result = self.advance_active_battle_by_internal(0, true, 0)?;
        self.authoring_time_zero_completed(battle_uuid)?;
        Ok(result)
    }

    fn is_final_boss_node(&self, node_id: MapNodeId) -> Result<bool, GameError> {
        let run = self.run_state()?;
        let is_standard_final_boss = run.run_progression.is_final_standard_floor()
            && matches!(
                run.map.terminal_resolution(node_id),
                Some(
                    crate::game::map::FacilityTerminalResolution::RunFinalBoss
                        | crate::game::map::FacilityTerminalResolution::StandardCompletion
                        | crate::game::map::FacilityTerminalResolution::SpecialRunCompletion
                )
            )
            && run
                .map
                .node(node_id)
                .is_some_and(|node| node.category == MapNodeCategory::Boss);
        Ok(is_standard_final_boss || run.boss_omen.forced_boss_node_id() == Some(node_id))
    }

    fn stage_committed_combat_resolution(
        &self,
        battle: &CombatBattleState,
    ) -> Result<StagedCombatResultState, GameError> {
        let mut staged = self.staged_combat_result_state()?;
        self.apply_post_battle_resolution_to_state(&mut staged, battle)?;
        Self::decrement_consumables_after_combat_node_in_roster(&mut staged.grant_state.roster);
        Ok(staged)
    }

    fn staged_combat_result_state(&self) -> Result<StagedCombatResultState, GameError> {
        Ok(StagedCombatResultState {
            grant_state: EconomyGrantState::capture(&self.state)?,
            condition_inventory_diff: InventoryDiffDto::default(),
            employee_died: false,
        })
    }

    fn commit_staged_combat_result_state(&mut self, staged: StagedCombatResultState) {
        if staged.employee_died {
            if let Some(run) = self.state.run.as_mut() {
                run.record_progress.record_employee_death();
            }
        }
        staged.grant_state.commit_into(&mut self.state);
    }

    fn validated_encounter_registrations(
        &self,
        battle: &CombatBattleState,
    ) -> Result<EncounterCompendiumProfileState, GameError> {
        if battle.verdict != crate::game::battle::types::BattleVerdict::Victory {
            return Err(GameError::InvalidAction);
        }
        let registrations =
            EncounterCompendiumProfileState::from_victory_candidates(&battle.compendium_candidates);
        registrations
            .validate_current_catalog_commit(self.game_data.as_ref())
            .map_err(GameError::InvalidStaticData)?;
        Ok(registrations)
    }

    fn commit_encounter_registrations(&mut self, registrations: EncounterCompendiumProfileState) {
        if self.state.run.as_ref().is_some_and(|run| run.run_progression.game_mode == crate::game::map::GameMode::Tutorial) { return; }
        self.state
            .player_profile
            .encounter_compendium
            .merge(registrations);
    }

    fn apply_post_battle_resolution_to_state(
        &self,
        staged: &mut StagedCombatResultState,
        battle: &CombatBattleState,
    ) -> Result<(), GameError> {
        let multiplier = if battle.verdict == crate::game::battle::types::BattleVerdict::Victory {
            battle.deterrence.victory_quantity_multiplier
        } else { 1 };
        let rewardless_story = self.game_data.pve_data.get_by_id(&battle.encounter_id)
            .is_some_and(|encounter| encounter.is_rewardless_story_encounter());
        let post_battle_survival_xp = if rewardless_story { 0 } else { self.run_policy().growth.post_battle_survival_xp }
            .checked_mul(multiplier).ok_or(GameError::InvalidAction)?;
        staged.grant_state.roster = battle.employee_conditions.clone();
        staged.employee_died = self.state.roster.iter().any(|before| {
            before.is_alive()
                && staged
                    .grant_state
                    .roster
                    .get(&before.uuid)
                    .is_some_and(|after| after.condition.is_dead())
        });
        staged.condition_inventory_diff = self.recover_assets_for_new_deaths(
            &mut staged.grant_state.roster,
            &mut staged.grant_state.inventory,
            &mut staged.grant_state.skill_fragments,
        )?;
        // A form's progress and presentation change in the same terminal
        // transaction. Publish the changed copy through the ordinary inventory
        // diff as well as subsequent inventory/employee snapshots.
        if let Some(resources) = &battle.typed_resource_result {
            for owned in staged.grant_state.inventory.equipments.iter() {
                let Some(forms) = self.game_data.equipment_attack_catalog().by_equipment
                    .get(&owned.meta.uuid).and_then(|p| p.victory_forms.as_ref()) else { continue; };
                if !resources.balance_changed(&forms.confirmed, crate::game::resource::ResourceOwner::EquipmentCopy(owned.instance_uuid)) { continue; }
                let item = InventoryItemDto::Equipment(self.equipment_item_dto(owned));
                if let Some(updated) = staged.condition_inventory_diff.updated.iter_mut().find(|item| item.uuid() == owned.instance_uuid) {
                    *updated = item;
                } else {
                    staged.condition_inventory_diff.updated.push(item);
                }
            }
        }
        if staged.grant_state.roster.is_empty() {
            return Ok(());
        }

        let survival_xp_employee_ids = battle
            .result_stats
            .employees
            .iter()
            .filter(|result| {
                result.survived
                    && staged
                        .grant_state
                        .roster
                        .get(&result.employee_uuid)
                        .is_some_and(|employee| employee.is_alive())
            })
            .map(|result| result.employee_uuid)
            .collect::<std::collections::BTreeSet<_>>();

        if post_battle_survival_xp > 0 {
            for employee_uuid in survival_xp_employee_ids {
                staged.grant_state.grant_effects(
                    &self.game_data,
                    &self.state.skill_fragment_policy,
                    &GrantExecutionContext {
                        selected_employee_id: Some(employee_uuid),
                        ..GrantExecutionContext::default()
                    },
                    &[RewardEffect::GrantExperience {
                        amount: post_battle_survival_xp,
                        target: ExperienceTargetPolicy::SelectedEmployee,
                    }],
                    self.run_seed ^ 0x4752_414e_5446_5853,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn recover_assets_for_new_deaths(
        &self,
        roster: &mut EmployeeRoster,
        inventory: &mut Inventory,
        fragments: &mut crate::game::skill_fragment::SkillFragmentInventory,
    ) -> Result<InventoryDiffDto, GameError> {
        let newly_dead_ids = roster
            .iter()
            .filter(|employee| {
                employee.condition.is_dead()
                    && self
                        .state
                        .roster
                        .get(&employee.uuid)
                        .is_some_and(|before| before.is_alive())
            })
            .map(|employee| employee.uuid)
            .collect::<Vec<_>>();
        let mut diff = InventoryDiffDto::default();
        for employee_uuid in newly_dead_ids {
            let updated_item_uuids = EquipmentLoadoutTransition::new(inventory, roster)
                .clear_employee_equipment(employee_uuid)?;
            if let Some(copy_uuid) = roster
                .get(&employee_uuid)
                .and_then(|employee| employee.skill_fragments.active_copy_uuid())
            {
                fragments.unequip_copy(
                    roster,
                    &self.game_data.skill_fragment_data,
                    copy_uuid,
                    employee_uuid,
                )?;
            }
            let employee = roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            employee.active_consumable_modifier = None;
            for item_uuid in updated_item_uuids {
                let owned = inventory
                    .equipments
                    .get_item(&item_uuid)
                    .ok_or(GameError::InventoryItemNotFound)?;
                diff.updated.push(InventoryItemDto::Equipment(
                    self.equipment_item_dto(owned),
                ));
            }
        }
        Ok(diff)
    }

    pub(super) fn decrement_consumables_after_combat_node_in_roster(roster: &mut EmployeeRoster) {
        for employee in roster.iter_mut() {
            employee.decrement_consumable_after_combat_node();
        }
    }

    fn employee_outcome_before(
        &self,
        employee_results: &[BattleResultEmployeeStatsDto],
    ) -> Result<HashMap<Uuid, EmployeeOutcomeBefore>, GameError> {
        let roster = &self.state.roster;
        let mut before = HashMap::new();
        for result in employee_results {
            if let Some(employee) = roster.get(&result.employee_uuid) {
                before.insert(
                    result.employee_uuid,
                    EmployeeOutcomeBefore {
                        physical_damage: employee.condition.physical_damage,
                        trauma: employee.condition.trauma,
                        experience: employee.experience,
                        alive: employee.is_alive(),
                    },
                );
            }
        }
        Ok(before)
    }

    fn employee_outcome_changes_from_roster(
        roster: &EmployeeRoster,
        before: HashMap<Uuid, EmployeeOutcomeBefore>,
        employee_results: &[BattleResultEmployeeStatsDto],
    ) -> Result<Vec<NodeOutcomeEmployeeChange>, GameError> {
        let mut changes = Vec::new();
        for result in employee_results {
            let Some(before) = before.get(&result.employee_uuid) else {
                continue;
            };
            let Some(employee) = roster.get(&result.employee_uuid) else {
                continue;
            };
            changes.push(NodeOutcomeEmployeeChange {
                employee_uuid: result.employee_uuid,
                survived: result.survived,
                became_incapacitated: result.became_incapacitated,
                physical_damage_before: before.physical_damage,
                physical_damage_after: employee.condition.physical_damage,
                trauma_before: before.trauma,
                trauma_after: employee.condition.trauma,
                experience_before: before.experience,
                experience_after: employee.experience,
                was_alive: before.alive,
                is_alive: employee.is_alive(),
            });
        }
        Ok(changes)
    }

    fn combat_experience_context_from_roster(
        roster: &EmployeeRoster,
        employee_results: &[BattleResultEmployeeStatsDto],
    ) -> GrantExecutionContext {
        let eligible_employee_ids = employee_results
            .iter()
            .filter(|result| {
                result.survived
                    && roster
                        .get(&result.employee_uuid)
                        .is_some_and(|employee| employee.is_alive())
            })
            .map(|result| result.employee_uuid)
            .collect::<Vec<_>>();

        GrantExecutionContext {
            combat_participant_employee_ids: Some(eligible_employee_ids),
            selected_employee_id: None,
        }
    }

    fn combat_node_outcome_summary(
        &self,
        battle: &CombatBattleState,
        mission_success: bool,
        employee_changes: Vec<NodeOutcomeEmployeeChange>,
        inventory_diff: InventoryDiffDto,
    ) -> Result<NodeOutcomeSummary, GameError> {
        self.combat_node_outcome_summary_with_resolution(
            battle,
            mission_success,
            battle.winner,
            false,
            employee_changes,
            inventory_diff,
        )
    }

    fn combat_node_outcome_summary_with_resolution(
        &self,
        battle: &CombatBattleState,
        mission_success: bool,
        winner: BattleWinner,
        retreated: bool,
        employee_changes: Vec<NodeOutcomeEmployeeChange>,
        inventory_diff: InventoryDiffDto,
    ) -> Result<NodeOutcomeSummary, GameError> {
        let session = self
            .state
            .node_session
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        Ok(NodeOutcomeSummary {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: session.category,
            mission_success,
            combat: Some(CombatOutcomeSummary {
                node_type: battle.node_type,
                mission_variant: battle.mission_variant,
                outcome_id: battle.outcome_id.clone(),
                verdict: battle.verdict,
                winner,
                retreated,
            }),
            employee_changes,
            inventory_diff,
        })
    }

    pub(super) fn current_run_failure_reason(&self) -> Result<Option<RunFailureReason>, GameError> {
        let roster = self.roster()?;
        Ok(Self::current_run_failure_reason_for_roster(roster))
    }

    fn current_run_failure_reason_for_roster(roster: &EmployeeRoster) -> Option<RunFailureReason> {
        if roster.is_empty() {
            return None;
        }

        if !roster.iter().any(|employee| employee.is_alive()) {
            return Some(RunFailureReason::NoLivingEmployees);
        }

        if !roster
            .iter()
            .any(|employee| employee.is_battle_deployable())
        {
            return Some(RunFailureReason::NoDeployableEmployees);
        }

        None
    }

    pub(super) fn can_complete_combat_result_locally(&self) -> bool {
        matches!(self.state.lifecycle.phase(), GameState::CombatResult { battle_uuid }
            if self.state.confirmed_combat_result.as_ref()
                .is_some_and(|receipt| receipt.battle_uuid == *battle_uuid))
    }

    fn recovery_path_for_completion(
        &self,
        completion: &CombatResultNodeCompletion,
        grant_state: &EconomyGrantState,
    ) -> (Vec<MapNodeId>, bool) {
        let Ok(run) = self.run_state() else { return (Vec::new(), false); };
        let (map, navigation, progression) = match completion {
            CombatResultNodeCompletion::NodeCompleted { map, progression, .. } =>
                (map, progression, &run.run_progression),
            CombatResultNodeCompletion::RunComplete { map, progression, run_progression, .. } =>
                (map, progression, run_progression),
            CombatResultNodeCompletion::FloorAdvanced { next_map, next_progression, run_progression, .. } =>
                (next_map, next_progression, run_progression),
        };
        self.recovery_path_in_context(RecoveryContext {
            map, navigation, progression,
            roster: &grant_state.roster, inventory: &grant_state.inventory,
            fragments: &grant_state.skill_fragments, recruitment: &run.recruitment,
            enkephalin: grant_state.enkephalin.amount,
        })
    }

    fn current_forward_recovery_path(&self) -> Result<(Vec<MapNodeId>, bool), GameError> {
        let context = self.current_recovery_context()?;
        let (allowed, path_exists) = self.recovery_path_in_context(context);
        // Continue has already entered a concrete recovery activity; it is no longer selectable.
        let active_path = matches!(self.state.lifecycle.phase(), GameState::InNode { node_id, .. }
            if context.navigation.current_node_id == Some(*node_id)
                && self.recovery_node_is_useful(context, *node_id, false));
        Ok((allowed, path_exists || active_path))
    }

    pub(super) fn selectable_map_node_ids(&self) -> Vec<MapNodeId> {
        let Some(run) = self.state.run.as_ref() else {
            return Vec::new();
        };
        let Ok(template) = self.facility_template_for_map(&run.map) else {
            return Vec::new();
        };
        let selectable = run.map_progression.selectable_node_ids(&run.map, &template);
        let Some(restriction) = run.combat_recovery_restriction.as_ref() else {
            return selectable;
        };
        selectable
            .into_iter()
            .filter(|node_id| restriction.allowed_node_ids.contains(node_id))
            .collect()
    }

    pub(super) fn has_combat_recovery_restriction(&self) -> bool {
        self.state
            .run
            .as_ref()
            .is_some_and(|run| run.combat_recovery_restriction.is_some())
    }

    fn set_combat_recovery_restriction(&mut self, mut allowed_node_ids: Vec<MapNodeId>) {
        allowed_node_ids.sort_by_key(|node_id| node_id.0.as_u128());
        allowed_node_ids.dedup();
        if let Some(run) = self.state.run.as_mut() {
            run.combat_recovery_restriction = Some(CombatRecoveryRestriction { allowed_node_ids });
        }
    }

    pub(super) fn finalize_combat_recovery_node_resolution(
        &mut self,
        completion: BehaviorResult,
    ) -> Result<BehaviorResult, GameError> {
        if !self.has_combat_recovery_restriction() {
            return Ok(completion);
        }
        match self.reassess_combat_recovery()? {
            Some(reason) => self.fail_run(reason),
            None => Ok(completion),
        }
    }

    /// One owner refreshes the restriction after actual mutations or a continued next-floor draft.
    pub(super) fn reassess_combat_recovery(&mut self) -> Result<Option<RunFailureReason>, GameError> {
        if let Some(run) = self.state.run.as_mut() {
            run.combat_recovery_restriction = None;
        }
        match self.current_run_failure_reason()? {
            Some(reason @ RunFailureReason::NoDeployableEmployees) => {
                let (allowed_node_ids, path_exists) = self.current_forward_recovery_path()?;
                if path_exists {
                    self.set_combat_recovery_restriction(allowed_node_ids);
                    Ok(None)
                } else { Ok(Some(reason)) }
            }
            other => Ok(other),
        }
    }

    pub(super) fn block_or_fail_undeployable_combat_selection(
        &mut self,
    ) -> Result<Option<BehaviorResult>, GameError> {
        let reason = self.current_run_failure_reason()?;
        self.block_or_fail_undeployable_combat_selection_for_reason(reason)
    }

    fn block_or_fail_undeployable_combat_selection_for_reason(
        &mut self,
        reason: Option<RunFailureReason>,
    ) -> Result<Option<BehaviorResult>, GameError> {
        match reason {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => {
                let (allowed_node_ids, path_exists) = self.current_forward_recovery_path()?;
                if path_exists {
                    self.set_combat_recovery_restriction(allowed_node_ids);
                    self.handle_request_map_data().map(Some)
                } else {
                    self.fail_run(RunFailureReason::NoDeployableEmployees)
                        .map(Some)
                }
            }
            Some(RunFailureReason::BossDefeated) => {
                self.fail_run(RunFailureReason::BossDefeated).map(Some)
            }
            None => Ok(None),
        }
    }

    pub(super) fn fail_run(&mut self, reason: RunFailureReason) -> Result<BehaviorResult, GameError> {
        self.fail_run_with_outcome(reason, None)
    }

    fn fail_run_with_outcome(
        &mut self,
        reason: RunFailureReason,
        outcome: Option<NodeOutcomeSummary>,
    ) -> Result<BehaviorResult, GameError> {
        self.fail_run_with_reasons(reason, None, outcome)
    }

    fn fail_run_with_reasons(
        &mut self,
        reason: RunFailureReason,
        contributing_reason: Option<RunFailureReason>,
        outcome: Option<NodeOutcomeSummary>,
    ) -> Result<BehaviorResult, GameError> {
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.active_battle = None;
        if self.state.run_checkpoint.payload.is_some()
            && self.state.run_checkpoint.loads_used
                < crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS
        {
            self.transition_to(GameState::AwaitingRunFailureDecision {
                reason,
                contributing_reason,
            })?;
            return Ok(BehaviorResult::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                outcome,
            });
        }
        self.commit_run_failure(reason, outcome)
    }

    fn commit_run_failure(
        &mut self,
        reason: RunFailureReason,
        outcome: Option<NodeOutcomeSummary>,
    ) -> Result<BehaviorResult, GameError> {
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.settle_expedition_terminal(false)?;
        self.state.active_battle = None;
        self.transition_to(GameState::RunFailed { reason })?;
        Ok(BehaviorResult::RunFailed { reason, outcome })
    }

    pub(super) fn handle_confirm_run_failure(&mut self) -> Result<BehaviorResult, GameError> {
        let GameState::AwaitingRunFailureDecision { reason, .. } = self.state.lifecycle.phase()
        else {
            return Err(GameError::InvalidAction);
        };
        let reason = *reason;
        self.commit_run_failure(reason, None)
    }

    fn fail_run_if_needed(&mut self) -> Result<Option<BehaviorResult>, GameError> {
        match self.current_run_failure_reason()? {
            Some(RunFailureReason::NoLivingEmployees) => {
                self.fail_run(RunFailureReason::NoLivingEmployees).map(Some)
            }
            Some(RunFailureReason::NoDeployableEmployees) => self
                .fail_run(RunFailureReason::NoDeployableEmployees)
                .map(Some),
            Some(RunFailureReason::BossDefeated) => {
                self.fail_run(RunFailureReason::BossDefeated).map(Some)
            }
            None => Ok(None),
        }
    }

    fn active_battle_advanced_result(
        active: &mut ActiveBattleSession,
        post_battle_policy: PostBattleResolutionPolicy,
        finished: bool,
        include_setup_snapshot: bool,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        active.refresh_tactical_resource();
        active.apply_live_signals_to_tactical_resource();
        active.reconcile_employee_attempt(post_battle_policy)?;
        active.commit_battle_projection()?;
        let battle_setup_snapshot =
            include_setup_snapshot.then(|| active.battle_setup_snapshot_dto());
        let battle_update =
            active.battle_update_dto_after_or_resync(after_seq, active.employee_roster())?;
        Ok(BehaviorResult::BattleAdvanced {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_setup_snapshot,
            battle_update,
            finished,
        })
    }

    fn combat_battle_state_from_live_result(
        active: &ActiveBattleSession,
        outcome_id: crate::game::battle::types::BattleOutcomeId,
        verdict: crate::game::battle::types::BattleVerdict,
        result_stats: BattleResultStatsDto,
        participant_results: Vec<ParticipantBattleResult>,
    ) -> Result<CombatBattleState, GameError> {
        let outcome_policy = active
            .outcome_policies
            .iter()
            .find(|policy| policy.outcome_id == outcome_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "active battle outcome '{}' has no node policy",
                    outcome_id.as_str()
                ))
            })?;
        if outcome_policy.verdict != verdict {
            return Err(GameError::InvalidStaticData(format!(
                "active battle outcome '{}' verdict does not match its node policy",
                outcome_id.as_str()
            )));
        }
        let winner = verdict.winner();
        Ok(CombatBattleState {
            bonus_reward_effects: active.battle.performance_bonus_effects(
                &active.encounter_id, verdict, result_stats.battle.duration_ms),
            deterrence: active.combat_preview.deterrence,
            credit_settlements: active.battle.credit_settlements()?,
            typed_resource_result: active.battle.capture_run_resources()?,
            runtime_facts: Some(active.battle.runtime_fact_snapshot()?),
            primary_abnormality_id: active.primary_abnormality_id.clone(),
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            abnormality_uuid: active.node_id.0,
            outcome_id,
            verdict,
            winner,
            #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
            event_log: active.battle.event_log.clone(),
            result_stats,
            reward_mode: if active.stress_combat.is_some() { crate::game::enums::RewardMode::ClaimAll } else { outcome_policy.reward_mode },
            rewards: if let Some(policy) = &active.stress_combat {
                let completed = active.battle.stress_round_status().ok_or(GameError::InvalidAction)?.1;
                if verdict == crate::game::battle::types::BattleVerdict::Victory {
                    vec![crate::game::reward::RewardOption { id: "stress_accumulated".into(), uuid: active.node_id.0, name: "Stress combat rewards".into(), description: String::new(), icon: String::new(), effects: policy.rewards_through(completed, 1)? }]
                } else { vec![] }
            } else { outcome_policy.rewards.clone() },
            grant_node_bonus: active.stress_combat.is_none() && outcome_policy.grant_node_bonus,
            advance_encounter_progression: active.stress_combat.is_none() && outcome_policy.advance_encounter_progression,
            participant_results,
            employee_conditions: active.employee_attempt.roster.clone(),
            compendium_candidates: active.battle.compendium_candidates(),
        })
    }

    pub(super) fn transition_finished_live_battle_to_result(
        &mut self,
        battle_uuid: Uuid,
        mut battle: CombatBattleState,
    ) -> Result<(), GameError> {
        // Freeze authored choices and final quantities once, before both the
        // durable result projection and its grant entitlement consume them.
        if battle.verdict == crate::game::battle::types::BattleVerdict::Victory {
            if let Some(hatched)=self.run_state()?.revival.nodes.get(&MapNodeId(battle.abnormality_uuid)) {
                let stage = hatched.stage_policy(&self.game_data.pve_data)?;
                if stage.encounter_id!=battle.encounter_id {return Err(GameError::InvalidAction);}
                battle.reward_mode=RewardMode::ChooseOne;
                battle.rewards=stage.equipment_reward_options(&self.game_data.equipment_data)?;
            }
            for reward in &mut battle.rewards {
                crate::game::combat_setup::rewards::apply_deterrence_quantity_multiplier(
                    &mut reward.effects,
                    battle.deterrence.victory_quantity_multiplier,
                )?;
            }
        }
        self.authoring_battle_finished(battle_uuid, battle.verdict)?;
        self.capture_authoring_terminal(
            battle_uuid,
            &battle.outcome_id,
            battle.verdict,
            battle.runtime_facts.clone(),
        )?;
        #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
        if let Err(error) =
            super::combat_trace::export_debug_combat_trace(self.run_seed, battle_uuid, &battle)
        {
            warn!("Ignoring non-authoritative debug combat trace export failure: {error:?}");
        }
        let runtime_facts = battle.runtime_facts.clone();
        self.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
        self.state.active_battle = None;
        let mut summary = self
            .get_selected_event_snapshot_dto()?
            .ok_or_else(|| invariant("combat_result_summary"))?;
        if let crate::game::behavior::SelectedEventSnapshotDto::CombatBattle {
            has_event_log, ..
        } = &mut summary
        {
            // The diagnostic trace above is separate from this durable result.
            *has_event_log = false;
        }
        let sealed_victory = match self.state.active_node_content.as_ref() {
            Some(ActiveNodeContent::CombatBattle(result)) => result.deterrence.seal_released
                && result.verdict == crate::game::battle::types::BattleVerdict::Victory,
            _ => false,
        };
        if sealed_victory {
            let id=self.state.node_session.as_ref().ok_or(GameError::InvalidAction)?.node_id;
            self.commit_golden_bough_victory(id)?;
        }
        let acknowledgement = self.settle_combat_result()?;
        let continuation_phase = self.state.lifecycle.snapshot();
        if let Some(continuation) = self.state.combat_reward_continuation.as_mut() {
            continuation.battle_uuid = battle_uuid;
        }
        self.state.confirmed_combat_result =
            Some(std::sync::Arc::new(super::state::ConfirmedCombatResult {
                battle_uuid,
                summary,
                acknowledgement: std::sync::Arc::new(acknowledgement),
                continuation_phase,
                runtime_facts,
            }));
        self.transition_to(GameState::CombatResult { battle_uuid })
    }

    #[cfg(test)]
    pub fn advance_active_battle_by(&mut self, delta_ms: u64) -> Result<BehaviorResult, GameError> {
        let after_seq = self
            .state
            .active_battle
            .as_ref()
            .map(|active| active.battle.event_log.floor_seq())
            .unwrap_or(0);
        self.advance_active_battle_by_after_seq(delta_ms, after_seq)
    }

    pub fn advance_active_battle_by_after_seq(
        &mut self,
        delta_ms: u64,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_application(
            super::application::ApplicationRoute::BattleElapsedAdvance,
            move |draft| draft.advance_active_battle_by_internal(delta_ms, false, after_seq),
        )
    }

    fn advance_active_battle_by_internal(
        &mut self,
        delta_ms: u64,
        include_setup_snapshot: bool,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        self.advance_active_battle_transaction(
            BattleAdvanceStep::Elapsed(delta_ms),
            include_setup_snapshot,
            after_seq,
        )?
        .ok_or(GameError::InvalidAction)
    }

    fn advance_active_battle_transaction(
        &mut self,
        step: BattleAdvanceStep,
        include_setup_snapshot: bool,
        after_seq: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        self.advance_active_battle_staged(step, include_setup_snapshot, after_seq)
    }

    fn advance_active_battle_staged(
        &mut self,
        step: BattleAdvanceStep,
        include_setup_snapshot: bool,
        after_seq: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        let post_battle_policy = self.run_policy().post_battle;
        let (mut completed_battle, trigger_context) = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or(GameError::InvalidAction)?;
            if let BattleAdvanceStep::ServerTick(raw_delta_ms) = step {
                if active
                    .battle
                    .advance_forced_pause_host_elapsed(raw_delta_ms)?
                {
                    return Ok(Some(Self::active_battle_advanced_result(
                        active,
                        post_battle_policy,
                        false,
                        include_setup_snapshot,
                        after_seq,
                    )?));
                }
            }
            let step = match step {
                BattleAdvanceStep::Elapsed(delta_ms) => BattleAdvanceStep::Elapsed(delta_ms),
                BattleAdvanceStep::NextEventBucket => BattleAdvanceStep::NextEventBucket,
                BattleAdvanceStep::ServerTick(raw_delta_ms) => {
                    let Some(delta_ms) = active.simulation_delta_for_tick(raw_delta_ms) else {
                        return Ok(None);
                    };
                    BattleAdvanceStep::Elapsed(delta_ms)
                }
            };
            active.ensure_battle_projection_commit_available()?;
            active.ensure_battle_update_cursor_valid(after_seq)?;
            let previous_battle_time_ms = active.execution.last_event_time_ms();
            let outcome = match step {
                BattleAdvanceStep::Elapsed(delta_ms) => active
                    .battle
                    .step_battle_execution_by_in_candidate(&mut active.execution, delta_ms),
                BattleAdvanceStep::NextEventBucket => active
                    .battle
                    .step_battle_execution_in_candidate(&mut active.execution),
                BattleAdvanceStep::ServerTick(_) => unreachable!(),
            }
            .map_err(|error| {
                crate::game::battle::runtime_control::trusted_failure(error, "active_battle_step")
            })?;
            active.reconcile_employee_attempt(post_battle_policy)?;
            let completed_battle = match outcome {
                BattleStepOutcome::Running => None,
                BattleStepOutcome::Finished(_) => {
                    let battle_result = active
                        .battle
                        .finalize_battle_execution_in_candidate(&mut active.execution)?;
                    Some(Self::combat_battle_state_from_live_result(
                        active,
                        battle_result.outcome_id,
                        battle_result.verdict,
                        battle_result.result_stats,
                        battle_result.participant_results,
                    )?)
                }
            };
            let current_battle_time_ms = active.execution.last_event_time_ms();
            let trigger_context = (
                active.battle_uuid,
                active.encounter_id.clone(),
                active.node_type,
                active.mission_variant,
                previous_battle_time_ms,
                current_battle_time_ms,
            );
            (completed_battle, trigger_context)
        };
        let (
            battle_uuid,
            encounter_id,
            node_type,
            mission_variant,
            previous_battle_time_ms,
            current_battle_time_ms,
        ) = trigger_context;
        if include_setup_snapshot {
            self.evaluate_visual_novel_triggers(
                super::visual_novel_triggers::VisualNovelTriggerEvent::BattleStarted {
                    battle_uuid,
                    encounter_id,
                    node_type,
                    mission_variant,
                },
            )?;
        }
        self.evaluate_visual_novel_triggers(
            super::visual_novel_triggers::VisualNovelTriggerEvent::BattleTimeReached {
                battle_uuid,
                previous_battle_time_ms,
                current_battle_time_ms,
            },
        )?;
        let mut result = {
            let active = self
                .state
                .active_battle
                .as_mut()
                .ok_or_else(|| invariant("advance_active_battle_context"))?;
            Self::active_battle_advanced_result(
                active,
                post_battle_policy,
                completed_battle.is_some(),
                include_setup_snapshot,
                after_seq,
            )?
        };
        if let Some(battle) = completed_battle.as_mut() {
            battle.employee_conditions = self
                .state
                .active_battle
                .as_ref()
                .ok_or_else(|| invariant("advance_active_battle_context"))?
                .employee_attempt
                .roster
                .clone();
        }
        self.attach_visual_novel_to_battle_result(&mut result)?;
        self.authoring_battle_advanced(
            battle_uuid,
            previous_battle_time_ms,
            current_battle_time_ms,
        )?;
        if let Some(battle) = completed_battle {
            self.transition_finished_live_battle_to_result(battle_uuid, battle)?;
        }
        Ok(Some(result))
    }

    #[cfg(test)]
    pub fn advance_active_battle_for_server_tick(
        &mut self,
        raw_delta_ms: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        let after_seq = self
            .state
            .active_battle
            .as_ref()
            .map(|active| active.battle.event_log.floor_seq())
            .unwrap_or(0);
        self.advance_active_battle_for_server_tick_after_seq(raw_delta_ms, after_seq)
    }

    pub fn advance_active_battle_for_server_tick_after_seq(
        &mut self,
        raw_delta_ms: u64,
        after_seq: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        self.execute_application(
            super::application::ApplicationRoute::BattleServerTick,
            move |draft| {
                draft.advance_active_battle_for_server_tick_after_seq_draft(raw_delta_ms, after_seq)
            },
        )
    }

    pub(crate) fn prepare_active_battle_for_host_tick(
        &mut self,
        raw_delta_ms: u64,
        after_seq: u64,
    ) -> Result<super::application::PreparedApplicationCommit<Option<BehaviorResult>>, GameError>
    {
        self.prepare_application_for_publication(
            super::application::ApplicationRoute::BattleServerTick,
            move |draft| {
                draft.advance_active_battle_for_server_tick_after_seq_draft(raw_delta_ms, after_seq)
            },
        )
    }

    fn advance_active_battle_for_server_tick_after_seq_draft(
        &mut self,
        raw_delta_ms: u64,
        after_seq: u64,
    ) -> Result<Option<BehaviorResult>, GameError> {
        if self
            .state
            .visual_novel
            .active
            .as_ref()
            .is_some_and(|session| session.blocking)
            && !self
                .state
                .active_battle
                .as_ref()
                .is_some_and(|active| active.battle.forced_pause_remaining_ms().is_some())
        {
            return Ok(None);
        }
        self.advance_active_battle_transaction(
            BattleAdvanceStep::ServerTick(raw_delta_ms),
            false,
            after_seq,
        )
    }

    pub fn advance_active_battle_to_next_event_bucket(
        &mut self,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_application(
            super::application::ApplicationRoute::BattleEventBucket,
            |draft| {
                let after_seq = draft
                    .state
                    .active_battle
                    .as_ref()
                    .map(|active| active.battle.event_log.floor_seq())
                    .unwrap_or(0);
                draft
                    .advance_active_battle_transaction(
                        BattleAdvanceStep::NextEventBucket,
                        false,
                        after_seq,
                    )?
                    .ok_or(GameError::InvalidAction)
            },
        )
    }

    pub(super) fn handle_request_battle_state(
        &mut self,
        since_seq: Option<u64>,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let battle_update = active.battle_update_dto_after(since_seq.unwrap_or(0), &roster)?;
        Ok(BehaviorResult::BattleState {
            battle_uuid: active.battle_uuid,
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            encounter_id: active.encounter_id.clone(),
            combat_preview: active.combat_preview.clone(),
            battle_update,
            finished: active.execution.is_finished(),
        })
    }

    pub(super) fn handle_recover_battle_setup_loss(
        &self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let battle_update =
            active.battle_update_dto_after_or_resync(after_seq, active.employee_roster())?;
        Ok(BehaviorResult::BattleAdvanced {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_setup_snapshot: Some(active.battle_setup_snapshot_dto()),
            battle_update,
            finished: active.execution.is_finished(),
        })
    }

    /// Host-side disconnect transition. Active battles remain process-local,
    /// but simulation progress stops until a later explicit ResumeBattle.
    /// Repeated disconnect/replacement signals are idempotent.
    pub fn auto_pause_active_battle_for_disconnect(&mut self) -> Result<bool, GameError> {
        self.execute_application(
            super::application::ApplicationRoute::BattleDisconnectPause,
            Self::auto_pause_active_battle_for_disconnect_draft,
        )
    }

    fn auto_pause_active_battle_for_disconnect_draft(&mut self) -> Result<bool, GameError> {
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let input_preparation=active.battle.input_plan_preparation_id();
        if input_preparation.is_none() && active.playback.paused
            && active
                .interaction
                .restore_playback
                .is_none_or(|state| state.paused)
            && active
                .interaction
                .settings_restore_playback
                .is_none_or(|state| state.paused)
        {
            return Ok(false);
        }
        active.ensure_battle_projection_commit_available()?;
        if let Some((unit_id,preparation))=input_preparation {
            active.battle.apply_live_command_in_candidate(BattleLiveCommand::CancelInputPlan{unit_id,preparation,disconnected:true,time_ms:active.execution.last_event_time_ms()})?;
            active.interaction.input_plan_restore_playback=None;
        }
        active.playback.paused = true;
        active.remember_manual_pause(true);
        active.commit_battle_projection()?;
        Ok(true)
    }

    pub(super) fn handle_pause_battle(
        &mut self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if active.playback.paused {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::PauseBattle,
                reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
            });
        }
        active.ensure_battle_projection_commit_available()?;
        active.playback.paused = true;
        active.remember_manual_pause(true);
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_resume_battle(
        &mut self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if !active.playback.paused {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::ResumeBattle,
                reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
            });
        }
        active.ensure_battle_projection_commit_available()?;
        if active.battle.input_plan_preparation_id().is_some(){return Err(GameError::InvalidAction);}
        active.battle.resume_disconnected_input_plan();
        active.playback.paused = false;
        active.remember_manual_pause(false);
        active.sync_selection_playback();
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_set_battle_speed(
        &mut self,
        speed: crate::game::behavior::BattlePlaybackSpeed,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if active.has_selection() || active.battle.input_plan_paused() {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::SetBattleSpeed,
                reason: crate::game::behavior::ActionDisabledReasonCode::SelectionActive,
            });
        }
        if active.playback.speed == speed {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::SetBattleSpeed,
                reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
            });
        }
        active.ensure_battle_projection_commit_available()?;
        active.playback.speed = speed;
        active.playback_delta_remainder = 0;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattlePlaybackChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    fn with_staged_tactical_world_mutation<T>(
        &mut self,
        after_seq: u64,
        roster: &EmployeeRoster,
        operation: impl FnOnce(&mut ActiveBattleSession) -> Result<T, GameError>,
        finish: impl FnOnce(
            &ActiveBattleSession,
            T,
            crate::game::behavior::LiveBattleUpdateDto,
        ) -> BehaviorResult,
    ) -> Result<BehaviorResult, GameError> {
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        active.battle.preflight_runtime_fact_clone()?;
        let mut staged = crate::authoring::clone_shape::controlled_clone(active)?;
        staged.battle.event_log.prepare_authoring_write()?;
        staged.ensure_battle_projection_commit_available()?;
        let outcome = operation(&mut staged)?;
        staged.commit_battle_projection()?;
        let battle_update = staged.battle_update_dto_after_or_resync(after_seq, roster)?;
        let mut result = finish(&staged, outcome, battle_update);
        let original = self
            .state
            .active_battle
            .replace(staged)
            .expect("active battle was checked before tactical staging");
        if let Err(error) = self.attach_visual_novel_to_battle_result(&mut result) {
            self.state.active_battle = Some(original);
            return Err(error);
        }
        Ok(result)
    }

    pub(super) fn handle_deploy_unit(
        &mut self,
        employee_uuid: Uuid,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let employee_disabled_reason = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .battle_deployment_disabled_reason();
        if let Some(reason) = employee_disabled_reason {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::DeployUnit,
                reason,
            });
        }
        let combat_preview = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .combat_preview
            .clone();
        self.validate_employee_deployment_cell(&combat_preview, employee_uuid, position)?;
        let tile_kind = match Self::deployment_zone_kind_for_cell(&combat_preview, position)
            .ok_or(GameError::OutOfBounds)?
        {
            DeploymentZoneKind::Ground => crate::game::battle::types::TacticalTileKind::Ground,
            DeploymentZoneKind::Platform => crate::game::battle::types::TacticalTileKind::Platform,
        };
        let draft = battle_unit_draft_for_employee(
            self.roster()?,
            self.inventory()?,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
            employee_uuid,
        )?;
        self.validate_defense_route_deployable_profile(&draft)?;
        let (base_deploy_cost, current_hp_policy) = {
            let active = self
                .state
                .active_battle
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let deployment = active
                .live_deployment
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let redeploy_lock = deployment.redeploy_locks.get(&employee_uuid);
            (
                redeploy_lock
                    .map(|lock| lock.deploy_cost)
                    .unwrap_or_else(|| active.tactical_resource.base_deploy_cost()),
                redeploy_lock.and_then(|lock| lock.current_hp_policy),
            )
        };
        let deploy_cost = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .deployment_cost_after_consumable(base_deploy_cost);
        if let Some(reason) = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .deployment_candidate_eligibility(employee_uuid, deploy_cost, None)
            .disabled_reason_code
        {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::DeployUnit,
                reason,
            });
        }
        let roster = self.roster()?.clone();
        let post_battle_policy = self.run_policy().post_battle;
        self.with_staged_tactical_world_mutation(
            after_seq,
            &roster,
            |active| {
                let deployment = active
                    .live_deployment
                    .as_mut()
                    .ok_or(GameError::InvalidAction)?;
                let instance_salt = deployment.next_instance_salt;
                let next_instance_salt = deployment.next_instance_salt.saturating_add(1);
                let time_ms = active.execution.last_event_time_ms();
                let outcome = active
                    .battle
                    .apply_live_command_with_source_command_id_in_candidate(
                        BattleLiveCommand::DeployPlayerUnit {
                            draft,
                            position,
                            tile_kind,
                            facing,
                            instance_salt,
                            time_ms,
                            current_hp_policy,
                        },
                        source_command_id,
                    )?;
                let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
                    return Err(invariant("deploy_command_outcome"));
                };
                deployment.next_instance_salt = next_instance_salt;
                let resource_before = active.tactical_resource.current();
                assert!(
                    active.tactical_resource.pay(deploy_cost),
                    "validated deployment must retain its tactical-resource payment"
                );
                active.battle.record_resource_fact(
                    crate::game::battle::runtime_facts::ResourceOwner::Side(
                        crate::game::enums::Side::Player,
                    ),
                    crate::game::battle::runtime_facts::FactResource::TacticalPoints,
                    Some(crate::game::battle::runtime_facts::CostReason::TacticalDeploy),
                    crate::game::battle::runtime_facts::RuntimeFactSource {
                        actor: Some(unit_id),
                        ..Default::default()
                    },
                    -i64::from(deploy_cost),
                    resource_before,
                    active.tactical_resource.current(),
                    time_ms,
                )?;
                deployment.redeploy_locks.remove(&employee_uuid);
                deployment.deployed_units.insert(
                    employee_uuid,
                    LiveBattleDeployedUnitState {
                        unit_instance_id: unit_id,
                        facing,
                    },
                );
                active.apply_live_signals_to_tactical_resource();
                active.reconcile_employee_attempt(post_battle_policy)?;
                Ok(unit_id)
            },
            |active, unit_id, battle_update| BehaviorResult::BattleUnitDeployed {
                battle_uuid: active.battle_uuid,
                encounter_id: active.encounter_id.clone(),
                node_type: active.node_type,
                mission_variant: active.mission_variant,
                battle_update,
                employee_uuid,
                unit_instance_id: unit_id,
            },
        )
    }

    fn validate_defense_route_deployable_profile(
        &self,
        draft: &crate::game::battle::types::BattleUnitDraft,
    ) -> Result<(), GameError> {
        let Some(active) = self.state.active_battle.as_ref() else {
            return Err(GameError::InvalidAction);
        };
        if !crate::game::combat_setup::mission_policy::CombatMissionPolicy::starts_as_live_battle(
            active.node_type,
            active.mission_variant,
        ) {
            return Ok(());
        }
        let BattleUnitSource::Employee(profile) = &draft.source else {
            return Ok(());
        };
        let Some(skill_id) = &profile.skill_id else {
            return Ok(());
        };
        let skill = self
            .game_data
            .skill_data
            .get_by_id(skill_id.as_str())
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "DefenseRoute player skill '{}' is missing from skill database",
                    skill_id
                ))
            })?;
        let _ = skill;
        Ok(())
    }

    pub fn deployment_range_preview_dto(
        &self,
        employee_uuid: Uuid,
        position: Position,
        facing: FacingDirection,
    ) -> Result<LiveBattleRangePreviewsDto, GameError> {
        if let Some(reason) = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .battle_deployment_disabled_reason()
        {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::RequestDeploymentRangePreview,
                reason,
            });
        }
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        self.validate_employee_deployment_cell(&active.combat_preview, employee_uuid, position)?;
        let draft = battle_unit_draft_for_employee(
            self.roster()?,
            self.inventory()?,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
            employee_uuid,
        )?;
        self.validate_defense_route_deployable_profile(&draft)?;
        let profile = draft.combat_profile(self.game_data.as_ref())?;
        Ok(range_previews_for_combat_profile(
            &profile,
            self.game_data.as_ref(),
            &active.battle.battlefield,
            position,
            facing,
        ))
    }

    pub(super) fn handle_request_deployment_range_preview(
        &self,
        employee_uuid: Uuid,
        position: Position,
    ) -> Result<BehaviorResult, GameError> {
        Ok(BehaviorResult::DeploymentRangePreview {
            result: DeploymentRangePreviewResultDto {
                employee_uuid,
                position,
                deploy_eligibility: self
                    .state
                    .active_battle
                    .as_ref()
                    .and_then(|active| active.live_deployment_dto(active.employee_roster()))
                    .and_then(|deployment| {
                        deployment
                            .unit_deploy_costs
                            .into_iter()
                            .find(|candidate| candidate.employee_uuid == employee_uuid)
                    })
                    .map(|candidate| candidate.eligibility)
                    .unwrap_or_else(|| {
                        crate::game::behavior::BattlePayloadEligibilityDto::disabled(
                            crate::game::behavior::ActionDisabledReasonCode::UnitUnavailable,
                        )
                    }),
                facings: DeploymentRangePreviewFacingsDto {
                    up: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Up,
                        )?,
                    },
                    right: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Right,
                        )?,
                    },
                    down: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Down,
                        )?,
                    },
                    left: DeploymentRangePreviewFacingDto {
                        range_previews: self.deployment_range_preview_dto(
                            employee_uuid,
                            position,
                            FacingDirection::Left,
                        )?,
                    },
                },
            },
        })
    }

    pub(super) fn handle_withdraw_unit(
        &mut self,
        employee_uuid: Uuid,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let post_battle_policy = self.run_policy().post_battle;
        self.with_staged_tactical_world_mutation(
            after_seq,
            &roster,
            |active| {
                let deployment = active
                    .live_deployment
                    .as_mut()
                    .ok_or(GameError::InvalidAction)?;
                let unit_id = deployment
                    .deployed_units
                    .get(&employee_uuid)
                    .map(|deployed| deployed.unit_instance_id)
                    .ok_or(GameError::UnitNotFound)?;
                let time_ms = active.execution.last_event_time_ms();
                let outcome = active
                    .battle
                    .apply_live_command_with_source_command_id_in_candidate(
                        BattleLiveCommand::WithdrawUnit { unit_id, time_ms },
                        source_command_id,
                    )?;
                if !matches!(outcome, BattleLiveCommandOutcome::UnitWithdrawn { .. }) {
                    return Err(invariant("withdraw_command_outcome"));
                }
                let current_hp_policy = active
                    .battle
                    .units
                    .get(&unit_id)
                    .map(|unit| {
                        BattleDeployCurrentHpPolicy::withdraw_redeploy(
                            unit.stats.current_health,
                            unit.stats.max_health,
                        )
                    })
                    .ok_or_else(|| invariant("withdraw_retained_unit"))?;
                deployment.deployed_units.remove(&employee_uuid);
                let deploy_cost = active.tactical_resource.redeploy_cost();
                deployment.redeploy_locks.insert(
                    employee_uuid,
                    LiveBattleRedeployState {
                        ready_at_ms: active
                            .execution
                            .last_event_time_ms()
                            .saturating_add(deployment.withdraw_redeploy_cooldown_ms),
                        deploy_cost,
                        current_hp_policy: Some(current_hp_policy),
                    },
                );
                active.apply_live_signals_to_tactical_resource();
                active.reconcile_employee_attempt(post_battle_policy)?;
                Ok(unit_id)
            },
            |active, unit_id, battle_update| BehaviorResult::BattleUnitWithdrawn {
                battle_uuid: active.battle_uuid,
                encounter_id: active.encounter_id.clone(),
                node_type: active.node_type,
                mission_variant: active.mission_variant,
                battle_update,
                employee_uuid,
                unit_instance_id: unit_id,
            },
        )
    }

    pub(super) fn handle_set_engagement_stance(
        &mut self,
        employee_uuid: Uuid,
        stance: crate::game::battle::types::EngagementStance,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        if let Some(reason) = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .battle_deployment_disabled_reason()
        {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::SetEngagementStance,
                reason,
            });
        }

        let roster = self.roster()?.clone();
        self.with_staged_tactical_world_mutation(
            after_seq,
            &roster,
            |active| {
                let unit_id = active
                    .live_deployment
                    .as_ref()
                    .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
                    .map(|deployed| deployed.unit_instance_id)
                    .ok_or(GameError::ActionUnavailable {
                        action: crate::game::behavior::ActionKind::SetEngagementStance,
                        reason: crate::game::behavior::ActionDisabledReasonCode::UnitUnavailable,
                    })?;
                let time_ms = active.execution.last_event_time_ms();
                if let Some(reason) = active
                    .battle
                    .stance_change_disabled_reason(unit_id, stance, time_ms)
                {
                    return Err(GameError::ActionUnavailable {
                        action: crate::game::behavior::ActionKind::SetEngagementStance,
                        reason,
                    });
                }

                let outcome = active
                    .battle
                    .apply_live_command_with_source_command_id_in_candidate(
                        BattleLiveCommand::SetEngagementStance {
                            unit_id,
                            stance,
                            time_ms,
                        },
                        source_command_id,
                    )?;
                if !matches!(
                    outcome,
                    BattleLiveCommandOutcome::EngagementStanceChanged {
                        unit_id: changed_unit_id,
                        stance: changed_stance,
                    } if changed_unit_id == unit_id && changed_stance == stance
                ) {
                    return Err(invariant("stance_command_outcome"));
                }
                Ok(unit_id)
            },
            |active, unit_id, battle_update| BehaviorResult::BattleEngagementStanceChanged {
                battle_uuid: active.battle_uuid,
                encounter_id: active.encounter_id.clone(),
                node_type: active.node_type,
                mission_variant: active.mission_variant,
                battle_update,
                employee_uuid,
                unit_instance_id: unit_id,
                stance,
            },
        )
    }

    pub(super) fn handle_tactical_relocate_unit(
        &mut self,
        employee_uuid: Uuid,
        destination: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        if let Some(reason) = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?
            .battle_deployment_disabled_reason()
        {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::TacticalRelocateUnit,
                reason,
            });
        }

        let relocation_cost = self.run_policy().tactical_relocation.cost;
        let roster = self.roster()?.clone();
        self.with_staged_tactical_world_mutation(
            after_seq,
            &roster,
            |active| {
                let unit_id = active
                    .live_deployment
                    .as_ref()
                    .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
                    .map(|deployed| deployed.unit_instance_id)
                    .ok_or(GameError::ActionUnavailable {
                        action: crate::game::behavior::ActionKind::TacticalRelocateUnit,
                        reason: crate::game::behavior::ActionDisabledReasonCode::UnitUnavailable,
                    })?;
                let tile_kind =
                    match Self::deployment_zone_kind_for_cell(&active.combat_preview, destination)
                        .ok_or(GameError::OutOfBounds)?
                    {
                        DeploymentZoneKind::Ground => {
                            crate::game::battle::types::TacticalTileKind::Ground
                        }
                        DeploymentZoneKind::Platform => {
                            crate::game::battle::types::TacticalTileKind::Platform
                        }
                    };
                let time_ms = active.execution.last_event_time_ms();
                active.battle.validate_tactical_relocation(
                    unit_id,
                    destination,
                    tile_kind,
                    facing,
                    time_ms,
                )?;
                if !active.tactical_resource.can_pay(relocation_cost) {
                    return Err(GameError::ActionUnavailable {
                        action: crate::game::behavior::ActionKind::TacticalRelocateUnit,
                        reason:
                            crate::game::behavior::ActionDisabledReasonCode::InsufficientResources,
                    });
                }

                let outcome = active
                    .battle
                    .apply_live_command_with_source_command_id_in_candidate(
                        BattleLiveCommand::TacticalRelocateUnit {
                            unit_id,
                            destination,
                            tile_kind,
                            facing,
                            time_ms,
                        },
                        source_command_id,
                    )?;
                if !matches!(
                    outcome,
                    BattleLiveCommandOutcome::UnitTacticallyRelocated {
                        unit_id: relocated_unit_id,
                    } if relocated_unit_id == unit_id
                ) {
                    return Err(invariant("relocate_command_outcome"));
                }
                let resource_before = active.tactical_resource.current();
                assert!(
                    active.tactical_resource.pay(relocation_cost),
                    "validated relocation must retain its tactical-resource payment"
                );
                active.battle.record_resource_fact(
                    crate::game::battle::runtime_facts::ResourceOwner::Side(
                        crate::game::enums::Side::Player,
                    ),
                    crate::game::battle::runtime_facts::FactResource::TacticalPoints,
                    Some(crate::game::battle::runtime_facts::CostReason::TacticalRelocation),
                    crate::game::battle::runtime_facts::RuntimeFactSource {
                        actor: Some(unit_id),
                        ..Default::default()
                    },
                    -i64::from(relocation_cost),
                    resource_before,
                    active.tactical_resource.current(),
                    time_ms,
                )?;
                if let Some(deployed) = active
                    .live_deployment
                    .as_mut()
                    .and_then(|deployment| deployment.deployed_units.get_mut(&employee_uuid))
                {
                    deployed.facing = facing;
                }
                Ok(unit_id)
            },
            |active, unit_id, battle_update| BehaviorResult::BattleUnitTacticallyRelocated {
                battle_uuid: active.battle_uuid,
                encounter_id: active.encounter_id.clone(),
                node_type: active.node_type,
                mission_variant: active.mission_variant,
                battle_update,
                employee_uuid,
                unit_instance_id: unit_id,
                destination,
                facing,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_tactical_swap_units(
        &mut self,
        first_employee_uuid: Uuid,
        second_employee_uuid: Uuid,
        first_facing: crate::game::battle::tile_range::FacingDirection,
        second_facing: crate::game::battle::tile_range::FacingDirection,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        use crate::game::behavior::{ActionDisabledReasonCode, ActionKind};

        if first_employee_uuid == second_employee_uuid {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalSwapUnits,
                reason: ActionDisabledReasonCode::AlreadyActive,
            });
        }
        for employee_uuid in [first_employee_uuid, second_employee_uuid] {
            if let Some(reason) = self
                .roster()?
                .get(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?
                .battle_deployment_disabled_reason()
            {
                return Err(GameError::ActionUnavailable {
                    action: ActionKind::TacticalSwapUnits,
                    reason,
                });
            }
        }

        let relocation_cost = self.run_policy().tactical_relocation.cost;
        let roster = self.roster()?.clone();
        self.with_staged_tactical_world_mutation(
            after_seq,
            &roster,
            |active| {
                let deployment = active
                    .live_deployment
                    .as_ref()
                    .ok_or(GameError::InvalidAction)?;
                let first_unit_id = deployment
                    .deployed_units
                    .get(&first_employee_uuid)
                    .map(|deployed| deployed.unit_instance_id)
                    .ok_or(GameError::ActionUnavailable {
                        action: ActionKind::TacticalSwapUnits,
                        reason: ActionDisabledReasonCode::UnitUnavailable,
                    })?;
                let second_unit_id = deployment
                    .deployed_units
                    .get(&second_employee_uuid)
                    .map(|deployed| deployed.unit_instance_id)
                    .ok_or(GameError::ActionUnavailable {
                        action: ActionKind::TacticalSwapUnits,
                        reason: ActionDisabledReasonCode::UnitUnavailable,
                    })?;
                let first_position = active
                    .battle
                    .unit_projected_tile(first_unit_id)
                    .ok_or(GameError::UnitNotFound)?;
                let second_position = active
                    .battle
                    .unit_projected_tile(second_unit_id)
                    .ok_or(GameError::UnitNotFound)?;
                let first_destination_tile_kind = match Self::deployment_zone_kind_for_cell(
                    &active.combat_preview,
                    second_position,
                )
                .ok_or(GameError::OutOfBounds)?
                {
                    DeploymentZoneKind::Ground => {
                        crate::game::battle::types::TacticalTileKind::Ground
                    }
                    DeploymentZoneKind::Platform => {
                        crate::game::battle::types::TacticalTileKind::Platform
                    }
                };
                let second_destination_tile_kind = match Self::deployment_zone_kind_for_cell(
                    &active.combat_preview,
                    first_position,
                )
                .ok_or(GameError::OutOfBounds)?
                {
                    DeploymentZoneKind::Ground => {
                        crate::game::battle::types::TacticalTileKind::Ground
                    }
                    DeploymentZoneKind::Platform => {
                        crate::game::battle::types::TacticalTileKind::Platform
                    }
                };
                let time_ms = active.execution.last_event_time_ms();
                active.battle.validate_tactical_swap(
                    first_unit_id,
                    second_unit_id,
                    first_destination_tile_kind,
                    second_destination_tile_kind,
                    time_ms,
                )?;
                if !active.tactical_resource.can_pay(relocation_cost) {
                    return Err(GameError::ActionUnavailable {
                        action: ActionKind::TacticalSwapUnits,
                        reason: ActionDisabledReasonCode::InsufficientResources,
                    });
                }

                let outcome = active
                    .battle
                    .apply_live_command_with_source_command_id_in_candidate(
                        BattleLiveCommand::TacticalSwapUnits {
                            first_unit_id,
                            second_unit_id,
                            first_destination_tile_kind,
                            second_destination_tile_kind,
                            first_facing,
                            second_facing,
                            time_ms,
                        },
                        source_command_id,
                    )?;
                if !matches!(
                    outcome,
                    BattleLiveCommandOutcome::UnitsTacticallySwapped {
                        first_unit_id: swapped_first,
                        second_unit_id: swapped_second,
                    } if swapped_first == first_unit_id && swapped_second == second_unit_id
                ) {
                    return Err(invariant("swap_command_outcome"));
                }
                let resource_before = active.tactical_resource.current();
                assert!(
                    active.tactical_resource.pay(relocation_cost),
                    "validated tactical swap must retain its tactical-resource payment"
                );
                active.battle.record_resource_fact(
                    crate::game::battle::runtime_facts::ResourceOwner::Side(
                        crate::game::enums::Side::Player,
                    ),
                    crate::game::battle::runtime_facts::FactResource::TacticalPoints,
                    Some(crate::game::battle::runtime_facts::CostReason::TacticalSwap),
                    crate::game::battle::runtime_facts::RuntimeFactSource {
                        actor: None,
                        ..Default::default()
                    },
                    -i64::from(relocation_cost),
                    resource_before,
                    active.tactical_resource.current(),
                    time_ms,
                )?;
                let deployment = active
                    .live_deployment
                    .as_mut()
                    .expect("validated tactical swap must retain live deployment");
                deployment
                    .deployed_units
                    .get_mut(&first_employee_uuid)
                    .expect("validated first swap employee must remain deployed")
                    .facing = first_facing;
                deployment
                    .deployed_units
                    .get_mut(&second_employee_uuid)
                    .expect("validated second swap employee must remain deployed")
                    .facing = second_facing;
                Ok((
                    first_unit_id,
                    second_unit_id,
                    first_position,
                    second_position,
                ))
            },
            |active,
             (first_unit_id, second_unit_id, first_position, second_position),
             battle_update| {
                BehaviorResult::BattleUnitsTacticallySwapped {
                    battle_uuid: active.battle_uuid,
                    encounter_id: active.encounter_id.clone(),
                    node_type: active.node_type,
                    mission_variant: active.mission_variant,
                    battle_update,
                    first_employee_uuid,
                    second_employee_uuid,
                    first_unit_instance_id: first_unit_id,
                    second_unit_instance_id: second_unit_id,
                    first_destination: second_position,
                    second_destination: first_position,
                    first_facing,
                    second_facing,
                }
            },
        )
    }

    pub(super) fn handle_begin_skill_targeting(
        &mut self,
        employee_uuid: Uuid,
        skill_id: SkillId,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        use crate::game::behavior::{ActionDisabledReasonCode, ActionKind};

        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        if active.tactical_focus.is_some() {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::BeginSkillTargeting,
                reason: ActionDisabledReasonCode::AlreadyActive,
            });
        }
        active.ensure_battle_projection_commit_available()?;
        let unit_id = active
            .live_deployment
            .as_ref()
            .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
            .map(|deployed| deployed.unit_instance_id)
            .ok_or(GameError::ActionUnavailable {
                action: ActionKind::BeginSkillTargeting,
                reason: ActionDisabledReasonCode::UnitUnavailable,
            })?;
        let time_ms = active.execution.last_event_time_ms();
        let selection = active
            .battle
            .validate_manual_skill_targeting_focus(time_ms, unit_id, &skill_id)
            .map_err(|error| match error {
                GameError::ActionUnavailable { reason, .. } => GameError::ActionUnavailable {
                    action: ActionKind::BeginSkillTargeting,
                    reason,
                },
                other => other,
            })?;
        let purpose = match selection {
            SkillCastSelection::PlayerUnit => LiveBattleTacticalFocusPurposeDto::PlayerUnitSkill,
            SkillCastSelection::PlayerTile => LiveBattleTacticalFocusPurposeDto::PlayerTileSkill,
            SkillCastSelection::PlayerDirection => LiveBattleTacticalFocusPurposeDto::PlayerDirectionSkill,
            SkillCastSelection::PlayerTilePair => LiveBattleTacticalFocusPurposeDto::PlayerTilePairSkill,
            _ => return Err(GameError::InvalidAction),
        };
        active.tactical_focus = Some(super::state::SkillTargetingFocus {
            purpose,
            caster_unit_instance_id: unit_id,
            skill_id,
        });
        active.sync_selection_playback();
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattleSkillTargetingFocusChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_cancel_skill_targeting(
        &mut self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        use crate::game::behavior::{ActionDisabledReasonCode, ActionKind};

        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let Some(_) = active.tactical_focus.as_ref() else {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::CancelSkillTargeting,
                reason: ActionDisabledReasonCode::ContextIrrelevant,
            });
        };
        active.ensure_battle_projection_commit_available()?;
        active.tactical_focus = None;
        active.sync_selection_playback();
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattleSkillTargetingFocusChanged {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
        })
    }

    pub(super) fn handle_preview_skill_target(
        &self,
        employee_uuid: Uuid,
        skill_id: SkillId,
        target: crate::game::battle::event_log::SkillCastTarget,
    ) -> Result<BehaviorResult, GameError> {
        use crate::game::behavior::ActionDisabledReasonCode;

        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let unit_id = active
            .live_deployment
            .as_ref()
            .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
            .map(|deployed| deployed.unit_instance_id)
            .ok_or(GameError::UnitNotFound)?;
        let focus_matches = active.tactical_focus.as_ref().is_some_and(|focus| {
            focus.caster_unit_instance_id == unit_id && focus.skill_id == skill_id
        });
        let eligibility = if !focus_matches {
            BattlePayloadEligibilityDto::disabled(ActionDisabledReasonCode::NoEligiblePayload)
        } else {
            match active.battle.validate_manual_skill_activation(
                active.execution.last_event_time_ms(),
                unit_id,
                &skill_id,
                Some(target),
            ) {
                Ok(()) => BattlePayloadEligibilityDto::enabled(),
                Err(GameError::ActionUnavailable { reason, .. }) => {
                    BattlePayloadEligibilityDto::disabled(reason)
                }
                Err(GameError::InvalidStaticData(_)) => BattlePayloadEligibilityDto::disabled(
                    ActionDisabledReasonCode::SkillUnavailable,
                ),
                Err(error) => return Err(error),
            }
        };
        let object_placements = if eligibility.enabled {
            active.battle.object_creation_preview(
                active.execution.last_event_time_ms(),
                unit_id,
                &skill_id,
                Some(target),
            )?
        } else {
            Vec::new()
        };
        Ok(BehaviorResult::SkillTargetPreview {
            result: SkillTargetPreviewResultDto {
                counter_channel: if eligibility.enabled {
                    match target {
                        crate::game::battle::event_log::SkillCastTarget::Tile { position } => active.battle.counter_channel_preview(unit_id, &skill_id, position, active.execution.last_event_time_ms())?,
                        _ => None,
                    }
                } else { None },
                object_placements,
                employee_uuid,
                unit_instance_id: unit_id,
                skill_id,
                target,
                battle_state_revision: active.battle_state_revision,
                eligibility,
            },
        })
    }

    pub(super) fn handle_activate_skill(
        &mut self,
        employee_uuid: Uuid,
        skill_id: crate::game::ability::SkillId,
        target: Option<crate::game::battle::event_log::SkillCastTarget>,
        activation: Option<u64>,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let post_battle_policy = self.run_policy().post_battle;
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.ensure_battle_projection_commit_available()?;
        let unit_id = active
            .live_deployment
            .as_ref()
            .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
            .map(|deployed| deployed.unit_instance_id)
            .ok_or(GameError::UnitNotFound)?;
        let time_ms = active.execution.last_event_time_ms();
        let selection = active
            .battle
            .current_skill_cast_selection(&skill_id, unit_id)?
            .ok_or_else(|| {
                GameError::InvalidCommandData(format!(
                    "manual skill '{}' is missing from skill database",
                    skill_id
                ))
            })?;
        let player_selected = activation.is_none()
            && matches!(
                selection,
                SkillCastSelection::PlayerUnit | SkillCastSelection::PlayerTile | SkillCastSelection::PlayerTilePair | SkillCastSelection::PlayerDirection
            );
        let focus_matches = active.tactical_focus.as_ref().is_some_and(|focus| {
            focus.caster_unit_instance_id == unit_id && focus.skill_id == skill_id
        });
        if player_selected != focus_matches || (!player_selected && active.tactical_focus.is_some())
        {
            return Err(GameError::ActionUnavailable {
                action: crate::game::behavior::ActionKind::ActivateSkill,
                reason: crate::game::behavior::ActionDisabledReasonCode::NoEligiblePayload,
            });
        }
        if activation.is_none() {
            active
                .battle
                .validate_manual_skill_activation(time_ms, unit_id, &skill_id, target)?;
        }
        let outcome = active
            .battle
            .apply_live_command_with_source_command_id_in_candidate(
                match activation {
                    Some(activation) => BattleLiveCommand::ReinputSustainedSkill {
                        unit_id,
                        skill_id: skill_id.clone(),
                        activation,
                        target,
                        time_ms,
                    },
                    None => BattleLiveCommand::ActivateSkill {
                        unit_id,
                        skill_id: skill_id.clone(),
                        target,
                        time_ms,
                    },
                },
                source_command_id,
            )?;
        if matches!(outcome,BattleLiveCommandOutcome::InputPlanPrepared{..}) {
            active.interaction.input_plan_restore_playback=Some(active.interaction.restore_playback.take().unwrap_or(active.playback));
            active.interaction.focused_unit_instance_id=None;active.interaction.deployment_employee_uuid=None;active.tactical_focus=None;
            active.playback.paused=true;active.playback_delta_remainder=0;
        } else if !matches!(outcome, BattleLiveCommandOutcome::SkillActivated { .. }) {return Err(invariant("skill_command_outcome"));}
        active.apply_live_signals_to_tactical_resource();
        if player_selected {
            active.tactical_focus = None;
            active.sync_selection_playback();
        }
        active.reconcile_employee_attempt(post_battle_policy)?;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattleSkillActivated {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
            employee_uuid,
            unit_instance_id: unit_id,
            skill_id,
        })
    }

    pub(super) fn handle_select_skill_relation(
        &mut self,
        employee_uuid: Uuid,
        entry: String,
        target: crate::game::battle::ids::UnitInstanceId,
        source_command_id: Option<&str>,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self.state.active_battle.as_mut().ok_or(GameError::InvalidAction)?;
        active.ensure_battle_projection_commit_available()?;
        let unit_id = active.live_deployment.as_ref()
            .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
            .map(|deployed| deployed.unit_instance_id)
            .ok_or(GameError::UnitNotFound)?;
        active.battle.apply_live_command_with_source_command_id_in_candidate(
            BattleLiveCommand::SelectRelation {
                unit_id,
                entry: entry.clone(),
                target,
                time_ms: active.execution.last_event_time_ms(),
            },
            source_command_id,
        )?;
        active.apply_live_signals_to_tactical_resource();
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattleRelationSelected {
            battle_uuid: active.battle_uuid,
            encounter_id: active.encounter_id.clone(),
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            battle_update,
            employee_uuid,
            unit_instance_id: unit_id,
            entry,
        })
    }

    pub(super) fn handle_input_plan_command(&mut self,battle_uuid:Uuid,revision:u64,employee_uuid:Uuid,input:Option<crate::game::ability::input_plan::ShotCommitInput>,cancel:Option<u64>,source_command_id:Option<&str>,after_seq:u64)->Result<BehaviorResult,GameError>{
        let roster=self.roster()?.clone();
        let active=self.state.active_battle.as_mut().ok_or(GameError::InvalidAction)?;
        if active.battle_uuid!=battle_uuid{return Err(GameError::InvalidAction);}
        let replay=input.as_ref().is_some_and(|i|active.battle.input_plan_employee_receipt(employee_uuid,i.preparation,i.request).is_some());
        if !replay && active.battle_state_revision!=revision{return Err(GameError::InvalidAction);}
        active.ensure_battle_projection_commit_available()?;
        let commit=input.is_some();let time_ms=active.execution.last_event_time_ms();
        if !replay {
            let unit_id=active.live_deployment.as_ref().and_then(|d|d.deployed_units.get(&employee_uuid)).map(|d|d.unit_instance_id).ok_or(GameError::UnitNotFound)?;
            let command=if let Some(input)=input {BattleLiveCommand::CommitInputPlan{unit_id,input,time_ms}}else{BattleLiveCommand::CancelInputPlan{unit_id,preparation:cancel.ok_or(GameError::InvalidAction)?,disconnected:false,time_ms}};
            active.battle.apply_live_command_with_source_command_id_in_candidate(command,source_command_id)?;
            let prior=active.interaction.input_plan_restore_playback.take().ok_or_else(||invariant("input_plan_playback_owner"))?;
            active.playback=prior;
            if commit {active.playback.paused=false;if prior.paused{active.playback.speed=crate::game::behavior::BattlePlaybackSpeed::X1;}}
            active.playback_delta_remainder=0;active.remember_manual_pause(active.playback.paused);
            active.apply_live_signals_to_tactical_resource();active.commit_battle_projection()?;
        }
        let battle_update=active.battle_update_dto_after_or_resync(after_seq,&roster)?;
        Ok(BehaviorResult::BattlePlaybackChanged{battle_uuid:active.battle_uuid,encounter_id:active.encounter_id.clone(),node_type:active.node_type,mission_variant:active.mission_variant,battle_update})
    }

    pub(super) fn handle_retreat_battle(&mut self) -> Result<BehaviorResult, GameError> {
        if !self.active_battle_retreat_available() {
            return Err(GameError::InvalidAction);
        }
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let node_id = active.node_id;
        let stress = active.stress_combat.is_some();
        let snapshot = crate::authoring::clone_shape::controlled_clone(
            active
                .entry_snapshot
                .as_ref()
                .ok_or_else(|| invariant("retreat_entry_snapshot"))?,
        )?;
        let retreats_used = self
            .run_state()?
            .abnormality_attempt_state(node_id)
            .retreats_used
            + 1;
        // Keep the active battle intact until the fallible shared copy succeeds.
        // The outer application draft remains authoritative for later failures.
        BattleEntrySnapshot::restore_shared(snapshot, &mut self.state)?;
        if stress { return self.handle_complete_node(); }
        self.run_state_mut()?.abnormality_attempts.insert(
            node_id,
            super::state::AbnormalityAttemptState {
                max_attempts: super::state::ABNORMALITY_MAX_ATTEMPTS,
                attempts_started: retreats_used,
                retreats_used,
            },
        );

        let (node_id, kind_id, category) = match self.state.lifecycle.phase() {
            GameState::NodeConfirm {
                node_id,
                kind_id,
                category,
            } => (*node_id, kind_id.clone(), *category),
            _ => return Err(invariant("retreat_restored_phase")),
        };
        let session = self
            .state
            .node_session
            .clone()
            .ok_or_else(|| invariant("retreat_restored_session"))?;
        let session = self.facility_session_view(session);
        let payload = session.payload.clone();
        let combat_preview = if matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss)
        {
            self.combat_preview_for_node(node_id)?
        } else {
            None
        };
        Ok(BehaviorResult::NodePreview {
            node_id,
            kind_id,
            category,
            payload,
            session,
            map: self.current_map_view()?,
            combat_preview,
            trauma_recovery_preview: None,
        })
    }

    pub(super) fn active_battle_retreat_available(&self) -> bool {
        self.state.active_battle.as_ref().is_some_and(|active| {
            !active.execution.is_finished()
                && !active.battle.stress_round_status().is_some_and(|(_,_,w)| w)
                && (active.stress_combat.is_some() || self
                    .state
                    .run
                    .as_ref()
                    .is_some_and(|run| run.abnormality_attempt_state(active.node_id).can_retreat()))
        })
    }

    pub(super) fn handle_complete_combat_result(&mut self) -> Result<BehaviorResult, GameError> {
        let receipt = self
            .state
            .confirmed_combat_result
            .take()
            .ok_or(GameError::InvalidAction)?;
        let next_phase = if self.state.combat_reward_continuation.is_some() {
            let reward = self
                .state
                .active_node_content
                .as_ref()
                .ok_or_else(|| invariant("combat_reward_content"))?
                .as_reward()?;
            GameState::InReward {
                reward_uuid: reward.stage_uuid,
            }
        } else {
            receipt.continuation_phase.clone()
        };
        self.transition_to(next_phase)?;
        if self.state.combat_reward_continuation.is_none()
            && matches!(self.state.lifecycle.phase(), GameState::RunComplete)
        {
            self.settle_expedition_terminal(false)?;
        }
        Ok(receipt.acknowledgement.as_ref().clone())
    }

    fn settle_combat_result(&mut self) -> Result<BehaviorResult, GameError> {
        let battle = {
            let content = self
                .state
                .active_node_content
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let battle = content.as_combat_battle()?;
            if let Some(result) = &battle.typed_resource_result {
                result.preflight_native_clone()?;
            }
            crate::authoring::clone_shape::controlled_clone(battle)?
        };
        let battle_node_id = self
            .state
            .node_session
            .as_ref()
            .map(|session| session.node_id)
            .ok_or(GameError::InvalidAction)?;

        if let Some(result) = &battle.typed_resource_result {
            result
                .apply(&mut self.run_state_mut()?.typed_resources)
                .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        }

        self.run_state_mut()?.credit_ledgers.apply(&battle.credit_settlements)?;
        let employee_before = self.employee_outcome_before(&battle.result_stats.employees)?;

        if battle.verdict == crate::game::battle::types::BattleVerdict::Defeat {
            let staged = self.stage_committed_combat_resolution(&battle)?;
            let employee_changes = Self::employee_outcome_changes_from_roster(
                &staged.grant_state.roster,
                employee_before,
                &battle.result_stats.employees,
            )?;
            let outcome = self.combat_node_outcome_summary(
                &battle,
                false,
                employee_changes,
                staged.condition_inventory_diff.clone(),
            )?;
            let failure_reason =
                Self::current_run_failure_reason_for_roster(&staged.grant_state.roster);
            let is_final_boss = self.is_final_boss_node(battle_node_id)?;
            info!(
                "Combat outcome '{}' committed Defeat; skipping rewards",
                battle.outcome_id.as_str()
            );
            self.run_state_mut()?
                .boss_omen
                .handle_source_failure(battle_node_id);
            self.resolve_failed_encounter_progression(battle_node_id, true)?;
            if is_final_boss || failure_reason == Some(RunFailureReason::NoLivingEmployees) {
                let reason = if is_final_boss {
                    RunFailureReason::BossDefeated
                } else {
                    RunFailureReason::NoLivingEmployees
                };
                self.commit_staged_combat_result_state(staged);
                self.consume_failed_combat_node_before_run_failure()?;
                let contributing_reason = failure_reason.filter(|derived| *derived != reason);
                let result =
                    self.fail_run_with_reasons(reason, contributing_reason, Some(outcome))?;
                return Ok(result);
            }

            let node_completion =
                CombatResultNodeCompletion::try_from(self.plan_complete_current_node(false)?)?;
            let recovery_restriction =
                (failure_reason == Some(RunFailureReason::NoDeployableEmployees)).then(|| {
                    let (allowed_node_ids, path_exists) = self.recovery_path_for_completion(
                        &node_completion,
                        &staged.grant_state,
                    );
                    PlannedCombatRecoveryRestriction {
                        allowed_node_ids,
                        path_exists,
                    }
                });
            self.commit_staged_combat_result_state(staged);
            let completion = self.commit_live_combat_result_node_completion(node_completion)?;
            if let Some(restriction) = recovery_restriction {
                if !restriction.path_exists {
                    return self.fail_run_with_outcome(
                        RunFailureReason::NoDeployableEmployees,
                        Some(outcome),
                    );
                }
                self.set_combat_recovery_restriction(restriction.allowed_node_ids);
            }
            if let BehaviorResult::NodeCompleted { .. } = completion {
                let map = self.current_map_view()?;
                return Ok(BehaviorResult::NodeCompleted {
                    map,
                    outcome: Some(outcome),
                });
            }
            return Ok(completion);
        }

        if let Some(result) = self.fail_run_if_needed()? {
            return Ok(result);
        }

        let encounter_victory = if battle.advance_encounter_progression || self.run_state()?.revival.nodes.contains_key(&battle_node_id) || self.game_data.pve_data.get_by_id(&battle.encounter_id).is_some_and(|e| e.is_rewardless_story_encounter()) {
            self.plan_victorious_encounter_progression(&battle, battle_node_id)?
        } else {
            EncounterVictoryPlan::default()
        };
        let planned = self.plan_player_victory_combat_result_completion(
            &battle,
            battle_node_id,
            employee_before,
            encounter_victory,
        )?;
        self.commit_planned_player_victory_combat_result_completion(planned)
    }

    /// A losing terminal battle consumes its node without taking the success
    /// floor/Standard-completion branch or granting its permanent success record.
    fn consume_failed_combat_node_before_run_failure(&mut self) -> Result<(), GameError> {
        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let node_id = progression
            .current_node_id
            .ok_or(GameError::InvalidAction)?;
        let template = self.facility_template_for_map(&map)?;
        progression
            .complete_current_node(&mut map, &template)
            .map_err(|_| GameError::InvalidAction)?;
        let run = self.run_state_mut()?;
        run.map = map;
        run.map_progression = progression;
        run.retire_completed_node_setup(node_id);
        Ok(())
    }

    fn resolve_failed_encounter_progression(
        &mut self,
        node_id: MapNodeId,
        attempts_exhausted: bool,
    ) -> Result<(), GameError> {
        if self.run_state()?.run_progression.game_mode != GameMode::Expedition {
            return Ok(());
        }
        if self.run_state()?.revival.nodes.contains_key(&node_id) {
            if attempts_exhausted { self.run_state_mut()?.revival.resolve(node_id, false)?; }
            return Ok(());
        }
        if !attempts_exhausted {
            return Ok(());
        }
        let encounter_id = {
            let run = self.run_state()?;
            let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
            let MapNodePayload::Encounter {
                encounter_id: Some(encounter_id),
            } = &node.payload
            else {
                return Ok(());
            };
            encounter_id.clone()
        };
        let Some(encounter) = self.game_data.pve_data.get_by_id(&encounter_id) else {
            return Err(GameError::MissingResource("PveEncounter"));
        };
        if encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::NormalBoss {
            if let Some(source_id) = self
                .game_data
                .pve_data
                .get_reward_source_by_encounter_id(&encounter_id)
                .map(|source| source.reward_source_id.clone())
            {
                self.run_state_mut()?
                    .normal_boss_cycle
                    .consume_assigned(&source_id);
            }
        }
        Ok(())
    }

    fn plan_victorious_encounter_progression(&mut self, battle: &CombatBattleState, node_id: MapNodeId) -> Result<EncounterVictoryPlan, GameError> {
        use crate::game::data::pve_data::PveEncounterClass;
        if self.run_state()?.run_progression.game_mode != GameMode::Expedition { return Ok(EncounterVictoryPlan::default()); }
        let mut progression = crate::authoring::clone_shape::controlled_clone(&self.run_state()?.encounter_progression)?;
        if let Some(hatched) = progression.revival.nodes.get(&node_id).cloned() {
            let source = self.game_data.pve_data.reward_sources.iter().find(|s| s.reward_source_id == hatched.reward_source_id).ok_or(GameError::InvalidAction)?;
            let policy = source.revival.as_ref().ok_or(GameError::InvalidAction)?;
            let stage = policy.stage(hatched.stage);
            if stage.encounter_id != battle.encounter_id { return Err(GameError::InvalidAction); }
            crate::authoring::control::charge_serialized(policy, "revival_victory_rewards")?;
            let rewards = policy.victory_rewards(hatched.stage);
            progression.revival.resolve(node_id, true)?;
            return Ok(EncounterVictoryPlan { progression_after: Some(progression), reward_effects: rewards });
        }
        let encounter = self.game_data.pve_data.get_by_id(&battle.encounter_id).ok_or(GameError::MissingResource("PveEncounter"))?;
        if !matches!(encounter.encounter_class, PveEncounterClass::Elite | PveEncounterClass::NormalBoss | PveEncounterClass::FinalBoss) {
            return Ok(EncounterVictoryPlan::default());
        }
        if encounter.is_rewardless_story_encounter() {
            if let Some(source) = self.game_data.pve_data.get_reward_source_by_encounter_id(&battle.encounter_id) {
                progression.normal_boss_cycle.consume_assigned(&source.reward_source_id);
            }
            return Ok(EncounterVictoryPlan { progression_after: Some(progression), ..Default::default() });
        }
        let plan = progression.plan_encounter_reward(&self.game_data.pve_data, &battle.encounter_id).map_err(GameError::InvalidStaticData)?;
        let effects = plan.effects.clone();
        progression.commit_reward_plan(&self.game_data.pve_data, &plan).map_err(GameError::InvalidStaticData)?;
        if encounter.encounter_class == PveEncounterClass::NormalBoss {
            if plan.first_completion && self.game_data.pve_data.get_reward_source_by_encounter_id(&battle.encounter_id).is_some_and(|s| s.revival.is_some()) {
                progression.revival.grant_first_egg(&plan.reward_source_id)?;
            }
            progression.normal_boss_cycle.consume_assigned(&plan.reward_source_id);
        }
        Ok(EncounterVictoryPlan { progression_after: Some(progression), reward_effects: effects })
    }

    fn plan_player_victory_combat_result_completion(
        &mut self,
        battle: &CombatBattleState,
        _battle_node_id: MapNodeId,
        employee_before: HashMap<Uuid, EmployeeOutcomeBefore>,
        mut encounter_victory: EncounterVictoryPlan,
    ) -> Result<PlannedPlayerVictoryCombatResultCompletion, GameError> {
        let node_completion = CombatResultNodeCompletion::try_from(
            self.plan_complete_current_node_with_encounter_progression(
                false,
                encounter_victory.progression_after.as_ref(),
            )?,
        )?;
        let encounter_registrations = self.validated_encounter_registrations(battle)?;
        let mut staged = self.stage_committed_combat_resolution(battle)?;

        match Self::current_run_failure_reason_for_roster(&staged.grant_state.roster) {
            Some(RunFailureReason::NoLivingEmployees) => {
                return Ok(PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                    staged,
                    reason: RunFailureReason::NoLivingEmployees,
                    encounter_registrations,
                });
            }
            Some(RunFailureReason::BossDefeated) => {
                return Ok(PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                    staged,
                    reason: RunFailureReason::BossDefeated,
                    encounter_registrations,
                });
            }
            Some(RunFailureReason::NoDeployableEmployees) | None => {}
        }

        encounter_victory.reward_effects.extend(battle.bonus_reward_effects.clone());
        crate::game::combat_setup::rewards::apply_deterrence_quantity_multiplier(
            &mut encounter_victory.reward_effects, battle.deterrence.victory_quantity_multiplier)?;
        let reward_session = self.build_reward_session(
            battle.abnormality_uuid,
            battle.reward_mode,
            battle.rewards.clone(),
            false,
        );
        let rewards_to_apply = match reward_session.mode {
            RewardMode::ClaimAll => reward_session.rewards.clone(),
            RewardMode::ChooseOne => Vec::new(),
        };
        let reward_stage_uuid = reward_session.stage_uuid;
        let pending_reward = (reward_session.mode == RewardMode::ChooseOne
            && !reward_session.rewards.is_empty())
        .then_some(reward_session);
        let experience_context = Self::combat_experience_context_from_roster(
            &staged.grant_state.roster,
            &battle.result_stats.employees,
        );
        let (reward_inventory_diff, skill_fragment_diffs, mut employee_experience_diffs) = self
            .apply_reward_options_with_context_to_state(
                reward_stage_uuid,
                &rewards_to_apply,
                &experience_context,
                &mut staged.grant_state,
            )?;
        let mut inventory_diff = staged.condition_inventory_diff.clone();
        Self::merge_inventory_diff(&mut inventory_diff, reward_inventory_diff);
        let mut skill_fragment_diffs = skill_fragment_diffs;
        if !encounter_victory.reward_effects.is_empty() {
            let granted = staged.grant_state.grant_effects(
                &self.game_data,
                &self.state.skill_fragment_policy,
                &GrantExecutionContext::default(),
                &encounter_victory.reward_effects,
                self.run_seed ^ 0xABAD_0BAD_ABAD_0BAD,
            )?;
            Self::merge_inventory_diff(&mut inventory_diff, granted.inventory_diff);
            skill_fragment_diffs.extend(granted.skill_fragment_diffs);
            employee_experience_diffs.extend(granted.employee_experience_diffs);
        }
        crate::authoring::control::poll("encounter_victory_rewards_staged")?;
        let enkephalin = staged.grant_state.enkephalin.amount;
        let employee_changes = Self::employee_outcome_changes_from_roster(
            &staged.grant_state.roster,
            employee_before,
            &battle.result_stats.employees,
        )?;
        let outcome = self.combat_node_outcome_summary(
            battle,
            true,
            employee_changes,
            inventory_diff.clone(),
        )?;
        let recovery_restriction =
            (Self::current_run_failure_reason_for_roster(&staged.grant_state.roster)
                == Some(RunFailureReason::NoDeployableEmployees))
            .then(|| {
                let (allowed_node_ids, path_exists) = self.recovery_path_for_completion(
                    &node_completion,
                    &staged.grant_state,
                );
                PlannedCombatRecoveryRestriction {
                    allowed_node_ids,
                    path_exists,
                }
            });
        Ok(PlannedPlayerVictoryCombatResultCompletion::RewardsGranted {
            staged,
            encounter_registrations,
            encounter_progression_after_victory: encounter_victory.progression_after,
            node_completion,
            enkephalin,
            inventory_diff,
            skill_fragment_diffs,
            employee_experience_diffs,
            outcome,
            recovery_restriction,
            pending_reward,
            experience_context,
        })
    }

    fn commit_planned_player_victory_combat_result_completion(
        &mut self,
        planned: PlannedPlayerVictoryCombatResultCompletion,
    ) -> Result<BehaviorResult, GameError> {
        match planned {
            PlannedPlayerVictoryCombatResultCompletion::RunFailure {
                staged,
                reason,
                encounter_registrations,
            } => {
                self.commit_staged_combat_result_state(staged);
                self.consume_failed_combat_node_before_run_failure()?;
                let result = self.fail_run(reason)?;
                self.commit_encounter_registrations(encounter_registrations);
                Ok(result)
            }
            PlannedPlayerVictoryCombatResultCompletion::RewardsGranted {
                staged,
                encounter_registrations,
                encounter_progression_after_victory,
                node_completion,
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                outcome,
                recovery_restriction,
                pending_reward,
                experience_context,
            } => {
                self.commit_staged_combat_result_state(staged);
                if let Some(progression) = encounter_progression_after_victory {
                    self.run_state_mut()?.encounter_progression = progression;
                }
                let mut completion =
                    self.commit_live_combat_result_node_completion(node_completion)?;
                if let Some(restriction) = recovery_restriction {
                    if !restriction.path_exists {
                        completion = self.fail_run(RunFailureReason::NoDeployableEmployees)?;
                    } else {
                        self.set_combat_recovery_restriction(restriction.allowed_node_ids);
                        if let BehaviorResult::NodeCompleted { .. } = completion {
                            completion = BehaviorResult::NodeCompleted {
                                map: self.current_map_view()?,
                                outcome: None,
                            };
                        }
                    }
                }
                self.commit_encounter_registrations(encounter_registrations);
                if let Some(reward) = pending_reward {
                    self.state.combat_reward_continuation =
                        Some(super::state::CombatRewardContinuation {
                            // The terminal transition binds the live battle identity.
                            battle_uuid: uuid::Uuid::nil(),
                            context: experience_context,
                            return_phase: self.state.lifecycle.snapshot(),
                        });
                    self.state.active_node_content = Some(ActiveNodeContent::Reward(reward));
                }
                Ok(BehaviorResult::CombatRewardsGranted {
                    enkephalin,
                    inventory_diff,
                    skill_fragment_diffs,
                    employee_experience_diffs,
                    outcome,
                    completion: Box::new(completion),
                })
            }
        }
    }
}

impl GameCore {
    pub(super) fn start_stress_combat(&mut self, node_id: MapNodeId) -> Result<BehaviorResult, GameError> {
        let id = self.facility_layout().and_then(|l| l.stress_combat.as_ref()).ok_or(GameError::InvalidAction)?.encounter_id.clone();
        let snapshot = BattleEntrySnapshot::capture(&self.state)?;
        self.handle_explicit_combat_node(node_id, MapNodeCategory::Combat, None, id, Some(snapshot))
    }

    pub(super) fn stress_combat_view(&self) -> Result<Option<crate::game::stress_combat::StressCombatView>, GameError> {
        let Some(active) = &self.state.active_battle else { return Ok(None); };
        let Some(policy) = &active.stress_combat else { return Ok(None); };
        let (current, completed, waiting) = active.battle.stress_round_status().ok_or(GameError::InvalidAction)?;
        let multiplier = active.combat_preview.deterrence.victory_quantity_multiplier;
        let pending_rewards = policy.rewards_through(completed, multiplier)?;
        let next_index = if waiting { completed } else { current };
        let next_rewards = policy.rewards_for_round(next_index, multiplier)?;
        Ok(Some(crate::game::stress_combat::StressCombatView { battle_uuid: active.battle_uuid, current_round: (current+1) as u8, completed_rounds: completed as u8, awaiting_choice: waiting, quantity_multiplier: multiplier, pending_rewards, next_rewards }))
    }

    pub(super) fn handle_stress_combat_action(&mut self, battle_uuid: Uuid, completed_rounds: u8, extract: bool) -> Result<BehaviorResult, GameError> {
        let active = self.state.active_battle.as_mut().ok_or(GameError::InvalidAction)?;
        let (_, completed, waiting) = active.battle.stress_round_status().ok_or(GameError::InvalidAction)?;
        if active.battle_uuid != battle_uuid || completed != usize::from(completed_rounds) || !waiting { return Err(GameError::InvalidAction); }
        if extract {
            let result = active.battle.finish_stress_extraction(&mut active.execution)?;
            let battle = Self::combat_battle_state_from_live_result(active, result.outcome_id, result.verdict, result.result_stats, result.participant_results)?;
            self.transition_finished_live_battle_to_result(battle_uuid, battle)?;
            self.state.confirmed_combat_result.as_ref().map(|r| r.acknowledgement.as_ref().clone()).ok_or(GameError::InvalidAction)
        } else {
            active.battle.continue_stress_round()?;
            self.advance_active_battle_by_internal(0, false, 0)
        }
    }
}
