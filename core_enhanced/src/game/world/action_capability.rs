use super::GameCore;
use crate::game::behavior::{
    ActionCapabilityDto, ActionDisabledReasonCode, ActionKind, GameError, MaintenanceOptionsDto,
    PlayerBehavior,
};
use crate::game::data::visual_novel_data::VisualNovelSceneNext;
use crate::game::enums::RewardMode;
use crate::game::map::HeadquartersContactOption;
use crate::game::resources::{ActiveNodeContent, GameState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommonActionEvaluation {
    Irrelevant,
    Disabled(ActionDisabledReasonCode),
    Enabled,
}

const MAINTENANCE_ACTIONS: [ActionKind; 5] = [
    ActionKind::SelectSkillFragmentForm,
    ActionKind::AwakenSkillFragment,
    ActionKind::DismantleSkillFragment,
    ActionKind::DismantleEquipment,
    ActionKind::EnhanceEquipment,
];

// This is observable projection order, not route ownership. ActionKind::descriptor owns
// membership and the debug parity checks below prevent this ordered view from drifting.
const COMMON_ACTION_PROJECTION_ORDER: [ActionKind; 49] = [
    ActionKind::StartNewGame,
    ActionKind::SelectStarterEmployees,
    ActionKind::UseConsumableItem,
    ActionKind::EquipItem,
    ActionKind::UnEquipItem,
    ActionKind::EquipSkillFragment,
    ActionKind::UnequipSkillFragment,
    ActionKind::TransferSkillFragment,
    ActionKind::SelectSkillFragmentForm,
    ActionKind::AwakenSkillFragment,
    ActionKind::DismantleSkillFragment,
    ActionKind::DismantleEquipment,
    ActionKind::EnhanceEquipment,
    ActionKind::EquipRelic,
    ActionKind::UnequipRelic,
    ActionKind::RequestRelicState,
    ActionKind::RequestRelicNumericPlan,
    ActionKind::RequestRelicEffectivePlan,
    ActionKind::PreviewRelicCorruption,
    ActionKind::CorruptRelic,
    ActionKind::PreviewRelicDismantle,
    ActionKind::DismantleRelic,
    ActionKind::RequestMapData,
    ActionKind::SelectMapNode,
    ActionKind::ConvertMapNodeWithMonolith,
    ActionKind::ConfirmEnterNode,
    ActionKind::CancelSelectedNode,
    ActionKind::CompleteNode,
    ActionKind::LoadRunCheckpoint,
    ActionKind::ConfirmRunFailure,
    ActionKind::AbandonRun,
    ActionKind::RecruitEmployee,
    ActionKind::RequestEmergencySupplies,
    ActionKind::OpenHeadquartersShop,
    ActionKind::PurchaseItem,
    ActionKind::SellItem,
    ActionKind::RefreshShop,
    ActionKind::ExitShop,
    ActionKind::SelectReward,
    ActionKind::ClaimReward,
    ActionKind::ExitReward,
    ActionKind::AdvanceEventScene,
    ActionKind::ResolveEventChoiceCheck,
    ActionKind::ChooseRelicCorruption,
    ActionKind::AdvanceVisualNovel,
    ActionKind::SelectVisualNovelChoice,
    ActionKind::SweepMasteredElite,
    ActionKind::CompleteMasteredEliteSweep,
    ActionKind::CompleteCombatResult,
];

impl GameCore {
    pub fn get_action_capabilities(&self) -> Vec<ActionCapabilityDto> {
        let maintenance_options = self
            .is_in_maintenance_node()
            .then(|| self.maintenance_options());
        let mut actions = ActionKind::ALL
            .iter()
            .copied()
            .filter(|action| action.route().projects_common_capability())
            .collect::<Vec<_>>();
        actions.sort_by_key(|action| {
            COMMON_ACTION_PROJECTION_ORDER
                .iter()
                .position(|candidate| candidate == action)
                .unwrap_or(usize::MAX)
        });
        debug_assert!(COMMON_ACTION_PROJECTION_ORDER
            .iter()
            .all(|action| action.route().projects_common_capability()));
        debug_assert!(actions
            .iter()
            .all(|action| COMMON_ACTION_PROJECTION_ORDER.contains(action)));
        actions
            .into_iter()
            .filter_map(|action| self.project_common_action(action, maintenance_options.as_ref()))
            .collect()
    }

    fn project_common_action(
        &self,
        action: ActionKind,
        maintenance_options: Option<&MaintenanceOptionsDto>,
    ) -> Option<ActionCapabilityDto> {
        match self.evaluate_common_action_with_maintenance_options(action, maintenance_options) {
            CommonActionEvaluation::Irrelevant => None,
            CommonActionEvaluation::Enabled => {
                Some(ActionCapabilityDto::enabled(action, action.scope()))
            }
            CommonActionEvaluation::Disabled(reason) => Some(ActionCapabilityDto::disabled(
                action,
                action.scope(),
                reason,
            )),
        }
    }

    fn evaluate_common_action(&self, action: ActionKind) -> CommonActionEvaluation {
        let maintenance_options = (MAINTENANCE_ACTIONS.contains(&action)
            && self.is_in_maintenance_node())
        .then(|| self.maintenance_options());
        self.evaluate_common_action_with_maintenance_options(action, maintenance_options.as_ref())
    }

    fn evaluate_common_action_with_maintenance_options(
        &self,
        action: ActionKind,
        maintenance_options: Option<&MaintenanceOptionsDto>,
    ) -> CommonActionEvaluation {
        if !matches!(self.state.lifecycle.phase(), GameState::InBattle { .. })
            && self
                .state
                .visual_novel
                .active
                .as_ref()
                .is_some_and(|session| session.blocking)
            && !matches!(
                action,
                ActionKind::AdvanceVisualNovel | ActionKind::SelectVisualNovelChoice
            )
        {
            return CommonActionEvaluation::Irrelevant;
        }
        match action {
            ActionKind::StartNewGame => {
                if matches!(self.state.lifecycle.phase(), GameState::NotStarted) {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Irrelevant
                }
            }
            ActionKind::SelectStarterEmployees => {
                if !matches!(
                    self.state.lifecycle.phase(),
                    GameState::SelectingStarterEmployees
                ) {
                    return CommonActionEvaluation::Irrelevant;
                }
                let required = self.run_policy().setup.starter_employee_count;
                if self.state.starter_candidates.len() >= required {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
                }
            }
            ActionKind::UseConsumableItem
            | ActionKind::EquipItem
            | ActionKind::UnEquipItem
            | ActionKind::EquipSkillFragment
            | ActionKind::UnequipSkillFragment
            | ActionKind::TransferSkillFragment => self.evaluate_safezone_inventory_action(action),
            ActionKind::SelectSkillFragmentForm
            | ActionKind::AwakenSkillFragment
            | ActionKind::DismantleSkillFragment
            | ActionKind::DismantleEquipment
            | ActionKind::EnhanceEquipment => {
                if !self.is_in_maintenance_node() {
                    return CommonActionEvaluation::Irrelevant;
                }
                if maintenance_options.is_some_and(|options| {
                    Self::maintenance_action_has_executable_payload(options, action)
                }) {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
                }
            }
            ActionKind::RequestRelicState
            | ActionKind::RequestRelicNumericPlan
            | ActionKind::RequestRelicEffectivePlan => {
                if self.state.run.is_some() {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Irrelevant
                }
            }
            ActionKind::EquipRelic
            | ActionKind::UnequipRelic
            | ActionKind::PreviewRelicCorruption
            | ActionKind::CorruptRelic => {
                if !self.is_loadout_action_context() {
                    CommonActionEvaluation::Irrelevant
                } else if self.state.relics.is_empty() {
                    CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
                } else {
                    CommonActionEvaluation::Enabled
                }
            }
            ActionKind::PreviewRelicDismantle | ActionKind::DismantleRelic => {
                if !self.is_in_maintenance_node() {
                    CommonActionEvaluation::Irrelevant
                } else if self.state.relics.is_empty() {
                    CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
                } else {
                    CommonActionEvaluation::Enabled
                }
            }
            ActionKind::RequestMapData => {
                if !matches!(
                    self.state.lifecycle.phase(),
                    GameState::ViewingMap
                        | GameState::NodeConfirm { .. }
                        | GameState::InNode { .. }
                ) {
                    return CommonActionEvaluation::Irrelevant;
                }
                if self.state.run.is_some() {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(
                        ActionDisabledReasonCode::ProgressionUnavailable,
                    )
                }
            }
            ActionKind::SelectMapNode => {
                if !matches!(
                    self.state.lifecycle.phase(),
                    GameState::ViewingMap | GameState::NodeConfirm { .. }
                ) {
                    return CommonActionEvaluation::Irrelevant;
                }
                let has_selectable_node = !self.selectable_map_node_ids().is_empty();
                if has_selectable_node {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
                }
            }
            ActionKind::ConvertMapNodeWithMonolith => {
                if !matches!(self.state.lifecycle.phase(), GameState::NodeConfirm { .. }) {
                    return CommonActionEvaluation::Irrelevant;
                }
                if self
                    .monolith_conversion_snapshot()
                    .is_some_and(|value| value.options.iter().any(|option| option.enabled))
                {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Irrelevant
                }
            }
            ActionKind::LoadRunCheckpoint => {
                if !matches!(
                    self.state.lifecycle.phase(),
                    GameState::ViewingMap
                        | GameState::NodeConfirm { .. }
                        | GameState::AwaitingRunFailureDecision { .. }
                ) {
                    return CommonActionEvaluation::Irrelevant;
                }
                if self.checkpoint_payload_is_loadable() {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(
                        ActionDisabledReasonCode::CheckpointUnavailable,
                    )
                }
            }
            ActionKind::ConfirmRunFailure => {
                if matches!(
                    self.state.lifecycle.phase(),
                    GameState::AwaitingRunFailureDecision { .. }
                ) {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Irrelevant
                }
            }
            ActionKind::AbandonRun => {
                if self.state.run.is_some()
                    && matches!(
                        self.state.lifecycle.phase(),
                        GameState::ViewingMap | GameState::NodeConfirm { .. }
                    )
                {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Irrelevant
                }
            }
            ActionKind::ConfirmEnterNode => {
                if !matches!(self.state.lifecycle.phase(), GameState::NodeConfirm { .. }) {
                    return CommonActionEvaluation::Irrelevant;
                }
                if let Some(evaluation) = self.mastered_elite_resolution_evaluation() {
                    return match evaluation.dto.direct_battle.disabled_reason_code {
                        None => CommonActionEvaluation::Enabled,
                        Some(reason) => CommonActionEvaluation::Disabled(reason),
                    };
                }
                if self.state.node_session.is_some() {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(
                        ActionDisabledReasonCode::ProgressionUnavailable,
                    )
                }
            }
            ActionKind::CancelSelectedNode => {
                if matches!(self.state.lifecycle.phase(), GameState::NodeConfirm { .. }) {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Irrelevant
                }
            }
            ActionKind::CompleteNode => self.evaluate_complete_node(),
            ActionKind::RecruitEmployee
            | ActionKind::RequestEmergencySupplies
            | ActionKind::OpenHeadquartersShop => self.evaluate_headquarters_action(action),
            ActionKind::PurchaseItem
            | ActionKind::SellItem
            | ActionKind::RefreshShop
            | ActionKind::ExitShop => self.evaluate_shop_action(action),
            ActionKind::SelectReward | ActionKind::ClaimReward | ActionKind::ExitReward => {
                self.evaluate_reward_action(action)
            }
            ActionKind::AdvanceEventScene
            | ActionKind::ResolveEventChoiceCheck
            | ActionKind::ChooseRelicCorruption => self.evaluate_event_action(action),
            ActionKind::AdvanceVisualNovel | ActionKind::SelectVisualNovelChoice => {
                self.evaluate_nonbattle_visual_novel_action(action)
            }
            ActionKind::SweepMasteredElite => {
                let Some(evaluation) = self.mastered_elite_resolution_evaluation() else {
                    return CommonActionEvaluation::Irrelevant;
                };
                match evaluation.dto.sweep.disabled_reason_code {
                    None => CommonActionEvaluation::Enabled,
                    Some(reason) => CommonActionEvaluation::Disabled(reason),
                }
            }
            ActionKind::CompleteMasteredEliteSweep => {
                let GameState::MasteredEliteSweepResult { sweep_uuid } =
                    self.state.lifecycle.phase()
                else {
                    return CommonActionEvaluation::Irrelevant;
                };
                if self
                    .staged_mastered_elite_sweep_result()
                    .is_some_and(|result| result.sweep_uuid == *sweep_uuid)
                {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(
                        ActionDisabledReasonCode::CompletionUnavailable,
                    )
                }
            }
            ActionKind::CompleteCombatResult => {
                if !matches!(self.state.lifecycle.phase(), GameState::CombatResult { .. }) {
                    return CommonActionEvaluation::Irrelevant;
                }
                if self.can_complete_combat_result_locally() {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(
                        ActionDisabledReasonCode::CompletionUnavailable,
                    )
                }
            }
            _ => CommonActionEvaluation::Irrelevant,
        }
    }

    fn evaluate_safezone_inventory_action(&self, action: ActionKind) -> CommonActionEvaluation {
        let relevant = if action == ActionKind::UseConsumableItem {
            self.is_consumable_use_context()
        } else {
            self.is_loadout_action_context()
        };
        if !relevant {
            return CommonActionEvaluation::Irrelevant;
        }

        let enabled = match action {
            ActionKind::UseConsumableItem => self.state.inventory.consumables.iter().any(|item| {
                self.state.roster.iter().any(|employee| {
                    self.validate_use_consumable_item_payload(item.meta.uuid, employee.uuid)
                        .is_ok()
                })
            }),
            ActionKind::EquipItem => self.state.inventory.equipments.iter().any(|item| {
                self.state.roster.iter().any(|employee| {
                    self.validate_equip_item_payload(item.instance_uuid, employee.uuid)
                        .is_ok()
                })
            }),
            ActionKind::UnEquipItem => self.state.inventory.equipments.iter().any(|item| {
                item.equipped_to.is_some_and(|employee_uuid| {
                    self.validate_unequip_item_payload(item.instance_uuid, employee_uuid)
                        .is_ok()
                })
            }),
            ActionKind::EquipSkillFragment => self.state.skill_fragments.copies().any(|copy| {
                self.state.roster.iter().any(|employee| {
                    self.validate_equip_skill_fragment_payload(employee.uuid, copy.copy_uuid)
                        .is_ok()
                })
            }),
            ActionKind::UnequipSkillFragment => self.state.roster.iter().any(|employee| {
                employee
                    .skill_fragments
                    .active_copy_uuid()
                    .is_some_and(|id| {
                        self.validate_unequip_skill_fragment_payload(employee.uuid, id)
                            .is_ok()
                    })
            }),
            ActionKind::TransferSkillFragment => self.state.skill_fragments.copies().any(|copy| {
                copy.equipped_to.is_some()
                    && self.state.roster.iter().any(|employee| {
                        self.validate_transfer_skill_fragment_payload(copy.copy_uuid, employee.uuid)
                            .is_ok()
                    })
            }),
            _ => false,
        };

        if enabled {
            CommonActionEvaluation::Enabled
        } else {
            CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
        }
    }

    fn evaluate_complete_node(&self) -> CommonActionEvaluation {
        if !matches!(self.state.lifecycle.phase(), GameState::InNode { .. }) {
            return CommonActionEvaluation::Irrelevant;
        }
        match self.state.active_node_content.as_ref() {
            Some(ActiveNodeContent::Event(_)) | Some(ActiveNodeContent::HeadquartersContact(_)) => {
                CommonActionEvaluation::Irrelevant
            }
            Some(ActiveNodeContent::Shop(_))
            | Some(ActiveNodeContent::Reward(_))
            | Some(ActiveNodeContent::CombatBattle(_)) => CommonActionEvaluation::Irrelevant,
            _ if self.state.node_session.is_some() => CommonActionEvaluation::Enabled,
            _ => CommonActionEvaluation::Disabled(ActionDisabledReasonCode::CompletionUnavailable),
        }
    }

    fn evaluate_headquarters_action(&self, action: ActionKind) -> CommonActionEvaluation {
        let Some(ActiveNodeContent::HeadquartersContact(headquarters)) =
            self.state.active_node_content.as_ref()
        else {
            return CommonActionEvaluation::Irrelevant;
        };
        let option = match action {
            ActionKind::RecruitEmployee => HeadquartersContactOption::RecruitEmployee,
            ActionKind::RequestEmergencySupplies => {
                HeadquartersContactOption::RequestEmergencySupplies
            }
            ActionKind::OpenHeadquartersShop => HeadquartersContactOption::OpenHeadquartersShop,
            _ => return CommonActionEvaluation::Irrelevant,
        };
        if !headquarters.options.contains(&option) {
            return CommonActionEvaluation::Irrelevant;
        }
        let has_recruitment_candidates = self
            .state
            .run
            .as_ref()
            .and_then(|run| run.recruitment.active_session())
            .is_some_and(|session| {
                session.node_id == headquarters.node_id
                    && !session.employee_definition_ids.is_empty()
            });
        if action == ActionKind::RecruitEmployee
            && (!has_recruitment_candidates
                || self.state.roster.len() >= crate::game::employee::MAX_EMPLOYEE_ROSTER_SIZE)
        {
            CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
        } else {
            CommonActionEvaluation::Enabled
        }
    }

    fn evaluate_shop_action(&self, action: ActionKind) -> CommonActionEvaluation {
        if !matches!(self.state.lifecycle.phase(), GameState::InShop { .. }) {
            return CommonActionEvaluation::Irrelevant;
        }
        let Some(ActiveNodeContent::Shop(shop)) = self.state.active_node_content.as_ref() else {
            return CommonActionEvaluation::Disabled(ActionDisabledReasonCode::ContentUnavailable);
        };
        let enabled = match action {
            ActionKind::PurchaseItem => shop
                .offers
                .iter()
                .any(|offer| self.shop_purchase_eligibility(offer).enabled),
            ActionKind::SellItem => self
                .shop_sale_candidates(shop)
                .iter()
                .any(|candidate| candidate.eligibility.enabled),
            ActionKind::RefreshShop => self.shop_refresh_eligibility(shop).enabled,
            ActionKind::ExitShop => self.state.node_session.is_some(),
            _ => false,
        };
        if enabled {
            CommonActionEvaluation::Enabled
        } else {
            CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
        }
    }

    fn evaluate_reward_action(&self, action: ActionKind) -> CommonActionEvaluation {
        if matches!(
            self.state.lifecycle.phase(),
            GameState::InRewardClaimed { .. }
        ) {
            return if action == ActionKind::ExitReward {
                CommonActionEvaluation::Enabled
            } else {
                CommonActionEvaluation::Irrelevant
            };
        }
        if !matches!(self.state.lifecycle.phase(), GameState::InReward { .. }) {
            return CommonActionEvaluation::Irrelevant;
        }
        let Some(ActiveNodeContent::Reward(reward)) = self.state.active_node_content.as_ref()
        else {
            return CommonActionEvaluation::Disabled(ActionDisabledReasonCode::ContentUnavailable);
        };
        match action {
            ActionKind::SelectReward if reward.mode == RewardMode::ChooseOne => {
                if reward.rewards.is_empty() {
                    CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
                } else {
                    CommonActionEvaluation::Enabled
                }
            }
            ActionKind::SelectReward => CommonActionEvaluation::Irrelevant,
            ActionKind::ClaimReward => {
                if reward.mode == RewardMode::ClaimAll || reward.selected_reward_uuid.is_some() {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(
                        ActionDisabledReasonCode::ProgressionUnavailable,
                    )
                }
            }
            ActionKind::ExitReward => {
                if reward.can_skip {
                    CommonActionEvaluation::Enabled
                } else {
                    CommonActionEvaluation::Disabled(
                        ActionDisabledReasonCode::ProgressionUnavailable,
                    )
                }
            }
            _ => CommonActionEvaluation::Irrelevant,
        }
    }

    fn evaluate_nonbattle_visual_novel_action(&self, action: ActionKind) -> CommonActionEvaluation {
        if matches!(self.state.lifecycle.phase(), GameState::InBattle { .. }) {
            return CommonActionEvaluation::Irrelevant;
        }
        let Some(session) = self.state.visual_novel.active.as_ref() else {
            return CommonActionEvaluation::Irrelevant;
        };
        let next =
            self.active_visual_novel_scene_next(&session.session_id, &session.current_scene_id);
        match (action, next) {
            (ActionKind::AdvanceVisualNovel, Ok(VisualNovelSceneNext::Choices { .. })) => {
                CommonActionEvaluation::Irrelevant
            }
            (
                ActionKind::SelectVisualNovelChoice,
                Ok(VisualNovelSceneNext::Choices { choices }),
            ) => {
                if choices.is_empty() {
                    CommonActionEvaluation::Disabled(ActionDisabledReasonCode::NoEligiblePayload)
                } else {
                    CommonActionEvaluation::Enabled
                }
            }
            (ActionKind::AdvanceVisualNovel, Ok(_)) => CommonActionEvaluation::Enabled,
            (ActionKind::SelectVisualNovelChoice, Ok(_)) => CommonActionEvaluation::Irrelevant,
            (_, Err(_)) => {
                CommonActionEvaluation::Disabled(ActionDisabledReasonCode::ContentUnavailable)
            }
            _ => CommonActionEvaluation::Irrelevant,
        }
    }

    fn evaluate_event_action(&self, action: ActionKind) -> CommonActionEvaluation {
        match self.evaluate_event_action_kind(action) {
            Some(Ok(())) => CommonActionEvaluation::Enabled,
            Some(Err(reason)) => CommonActionEvaluation::Disabled(reason),
            None => CommonActionEvaluation::Irrelevant,
        }
    }

    /// Returns true when the behavior belongs to the migrated common capability slice.
    pub(super) fn preflight_common_capability_behavior(
        &self,
        behavior: &PlayerBehavior,
    ) -> Result<bool, GameError> {
        let action = behavior.kind();
        if matches!(self.state.lifecycle.phase(), GameState::InBattle { .. })
            && matches!(
                action,
                ActionKind::AdvanceVisualNovel | ActionKind::SelectVisualNovelChoice
            )
        {
            return Ok(false);
        }

        if !action.route().uses_common_preflight() {
            return Ok(false);
        }

        if let PlayerBehavior::ResolveEventChoiceCheck {
            node_id,
            event_id,
            current_scene_id,
            choice_id,
            employee_uuid,
        } = behavior
        {
            if self.is_exact_committed_event_resolve(
                *node_id,
                event_id,
                current_scene_id,
                choice_id,
                *employee_uuid,
            ) {
                return Ok(true);
            }
        }

        if let PlayerBehavior::ChooseRelicCorruption {
            node_id,
            event_id,
            current_scene_id,
            copy_uuid,
        } = behavior
        {
            if self.is_exact_committed_relic_corruption(
                *node_id,
                event_id,
                current_scene_id,
                *copy_uuid,
            )? {
                return Ok(true);
            }
        }

        match self.evaluate_common_action(action) {
            CommonActionEvaluation::Enabled => {}
            CommonActionEvaluation::Disabled(reason) => {
                return Err(GameError::ActionUnavailable { action, reason });
            }
            CommonActionEvaluation::Irrelevant => {
                return Err(GameError::ActionUnavailable {
                    action,
                    reason: ActionDisabledReasonCode::ContextIrrelevant,
                });
            }
        }

        if let PlayerBehavior::SelectMapNode { node_id } = behavior {
            self.validate_select_map_node(*node_id)
                .map_err(|_| GameError::ActionUnavailable {
                    action,
                    reason: ActionDisabledReasonCode::NoEligiblePayload,
                })?;
        }

        match behavior {
            PlayerBehavior::AdvanceEventScene {
                node_id,
                event_id,
                current_scene_id,
            } => {
                self.validate_event_advance_payload(*node_id, event_id, current_scene_id)
                    .map_err(|reason| GameError::ActionUnavailable { action, reason })?;
            }
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id,
                event_id,
                current_scene_id,
                choice_id,
                employee_uuid,
            } => {
                self.validate_event_resolve_payload(
                    *node_id,
                    event_id,
                    current_scene_id,
                    choice_id,
                    *employee_uuid,
                )
                .map_err(|reason| GameError::ActionUnavailable { action, reason })?;
            }
            PlayerBehavior::ChooseRelicCorruption {
                node_id,
                event_id,
                current_scene_id,
                copy_uuid,
            } => {
                let eligibility = self.validate_relic_corruption_payload(
                    *node_id,
                    event_id,
                    current_scene_id,
                    *copy_uuid,
                );
                crate::authoring::control::poll("relic_event_payload_preflight")?;
                eligibility.map_err(|reason| GameError::ActionUnavailable { action, reason })?;
            }
            _ => {}
        }

        Ok(true)
    }
}
