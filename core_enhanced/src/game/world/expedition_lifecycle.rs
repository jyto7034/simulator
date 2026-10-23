use super::{GameCore, GameCoreState};
use crate::game::{
    behavior::{BehaviorResult, GameError},
    expedition_lifecycle::{FirstReturnStory, VaultEquipment, VaultFragment},
    managers::uuid_manager::UuidManager,
    map::{GameMode, RunProgression},
    resources::GameState,
    skill_fragment::OwnedFragment,
};
use uuid::Uuid;

impl GameCore {
    pub(super) fn handle_return_from_expedition(&mut self, selected: &[crate::game::material_economy::MaterialRecovery]) -> Result<BehaviorResult, GameError> {
        use crate::game::data::equipment_data::EquipmentMaterialType;
        let run = self.run_state()?;
        let eligible = run.run_progression.pending_continuation.is_some_and(|pending| pending.depth >= 3)
            || (self.facility_gate_reached() && run.run_progression.floor_index() >= 2);
        if run.run_progression.game_mode != GameMode::Expedition || !eligible { return Err(GameError::InvalidAction); }
        let mut warehouse = self.state.player_profile.lifecycle.vault.materials.clone();
        let mut seen = std::collections::BTreeSet::new();
        let (mut parts, mut catalysts) = (0_u32, 0_u32);
        for row in selected {
            if row.amount == 0 || !seen.insert(&row.material_id) || self.state.inventory.equipment_materials.amount(&row.material_id) < row.amount {
                return Err(GameError::InvalidAction);
            }
            let kind = self.game_data.equipment_data.get_material_by_id(&row.material_id).ok_or(GameError::InvalidAction)?.material_type;
            let total = match kind { EquipmentMaterialType::AbnormalityPart => &mut parts, EquipmentMaterialType::RevivalCatalyst => &mut catalysts, _ => return Err(GameError::InvalidAction) };
            *total = total.checked_add(row.amount).ok_or(GameError::InvalidAction)?;
            let balance = warehouse.entry(row.material_id.clone()).or_default();
            *balance = balance.checked_add(row.amount).ok_or(GameError::InvalidAction)?;
        }
        if parts > 2 || catalysts > 1 { return Err(GameError::InvalidAction); }
        let result = self.handle_choose_expedition_continuation(crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Return)?;
        self.state.player_profile.lifecycle.vault.materials = warehouse;
        Ok(result)
    }

    pub(super) fn prepare_departure_identity(&mut self, mode: GameMode) -> Result<RunProgression, GameError> {
        let player = self.state.player.as_ref().ok_or(GameError::InvalidAction)?.id;
        let progression = RunProgression::new(self.run_seed, mode, self.run_policy().setup.standard_floor_count);
        let mut progression = match mode {
            GameMode::Expedition => {
                let instance = self.state.player_profile.lifecycle.next_instance(player)
                    .map_err(GameError::InvalidStaticData)?;
                self.state.player_profile.lifecycle.expedition_sequence = instance.sequence;
                self.state.player_profile.lifecycle.active_expedition = Some(instance);
                progression.with_expedition_instance(instance)
            }
            GameMode::Tutorial => progression.with_tutorial_identity(player),
        };
        if mode == GameMode::Expedition { progression.deterrence = self.run_policy().expedition_deterrence_initial; }
        self.state.uuid_manager = UuidManager::for_instance(self.run_seed, progression.instance_namespace);
        Ok(progression)
    }

    pub(super) fn take_departure_loans(&mut self, ids: &[Uuid]) -> Result<(), GameError> {
        if ids.len() > 2 || ids.iter().copied().collect::<std::collections::BTreeSet<_>>().len() != ids.len() {
            return Err(GameError::InvalidAction);
        }
        self.state.player_profile.lifecycle.vault.validate().map_err(GameError::InvalidStaticData)?;
        for id in ids {
            let vault = &self.state.player_profile.lifecycle.vault;
            if let Some(copy) = vault.equipment.get(id) {
                let metadata = self.game_data.equipment_data.get_by_id(&copy.definition_id)
                    .ok_or(GameError::InvalidAction)?;
                if metadata.equipment_type != copy.equipment_type { return Err(GameError::InvalidAction); }
            } else if let Some(copy) = vault.fragments.get(id) {
                if self.game_data.skill_fragment_data.get_by_id(&copy.definition_id).is_none() {
                    return Err(GameError::InvalidAction);
                }
            } else { return Err(GameError::InvalidAction); }
        }
        for id in ids {
            if let Some(copy) = self.state.player_profile.lifecycle.vault.equipment.remove(id) {
                let metadata = self.game_data.equipment_data.get_by_id(&copy.definition_id)
                    .ok_or(GameError::InvalidAction)?;
                let mut owned = crate::game::resources::inventory::OwnedEquipment::new(
                    copy.copy_uuid, std::sync::Arc::new(metadata.clone()),
                );
                owned.enhancement_level = copy.enhancement_level;
                owned.weapon_stage = copy.weapon_stage;
                self.state.inventory.equipments.add_item(owned).map_err(GameError::InvalidStaticData)?;
            } else {
                let copy = self.state.player_profile.lifecycle.vault.fragments.remove(id).ok_or(GameError::InvalidAction)?;
                self.state.skill_fragments.import_owned_copy(OwnedFragment {
                    copy_uuid: copy.copy_uuid, definition_id: copy.definition_id,
                    unlocked_forms: copy.unlocked_forms, selected_form: copy.selected_form,
                    equipped_to: None, investment: copy.investment,
                }, &self.game_data.skill_fragment_data, &self.state.skill_fragment_policy)?;
            }
        }
        self.validate_profile_active_ownership()
    }

    pub(super) fn validate_profile_active_ownership(&self) -> Result<(), GameError> {
        self.state.player_profile.lifecycle.vault.validate_active_ownership(
            &self.state.inventory, &self.state.skill_fragments,
        ).map_err(GameError::InvalidStaticData)
    }

    /// Settlement runs only after the command owner validates the terminal cause.
    pub(super) fn settle_expedition_terminal(&mut self, success: bool) -> Result<(), GameError> {
        let progression = self.run_state()?.run_progression.clone();
        self.validate_profile_active_ownership()?;
        if progression.game_mode == GameMode::Expedition {
            let instance = progression.expedition_instance.ok_or(GameError::InvalidAction)?;
            if self.state.player_profile.lifecycle.active_expedition != Some(instance) {
                return Err(GameError::InvalidAction);
            }
            if success {
                let equipment = self.state.inventory.equipments.iter().map(VaultEquipment::from_returned)
                    .collect::<Vec<_>>();
                let fragments = self.state.skill_fragments.copies().map(VaultFragment::from).collect::<Vec<_>>();
                let vault = &mut self.state.player_profile.lifecycle.vault;
                for copy in equipment {
                    if vault.equipment.insert(copy.copy_uuid, copy).is_some() { return Err(GameError::InvalidAction); }
                }
                for copy in fragments {
                    if vault.fragments.insert(copy.copy_uuid, copy).is_some() { return Err(GameError::InvalidAction); }
                }
                if self.state.player_profile.lifecycle.first_return_story == FirstReturnStory::Never {
                    self.state.player_profile.lifecycle.first_return_story = FirstReturnStory::Pending;
                }
            }
            self.state.player_profile.lifecycle.active_expedition = None;
            self.state.player_profile.lifecycle.warehouse_debits.clear();
        }
        self.run_state_mut()?.run_progression.pending_continuation = None;
        self.state.roster.clear();
        self.state.inventory = Default::default();
        self.state.skill_fragments = Default::default();
        self.state.relics = Default::default();
        self.state.crafting_resources = Default::default();
        self.state.run_checkpoint = Default::default();
        self.state.continue_state = Default::default();
        self.state.active_battle = None;
        self.state.confirmed_combat_result = None;
        self.state.combat_reward_continuation = None;
        self.state.player_profile.validate_persisted().map_err(GameError::InvalidStaticData)
    }

    pub(super) fn validate_pending_expedition_selection(
        &self, progression: &RunProgression, map: &crate::game::map::RunMap,
        navigation: &crate::game::map::MapProgression, phase: &GameState,
        session: Option<&crate::game::map::NodeSession>,
    ) -> Result<(), GameError> {
        use crate::game::expedition_lifecycle::ExpeditionContinuationCause as Cause;
        let Some(pending) = progression.pending_continuation else {
            return if session.is_none() { Ok(()) } else { Err(GameError::InvalidAction) };
        };
        let node = map.node(pending.node_id).ok_or(GameError::InvalidAction)?;
        let template = self.facility_template_for_map(map)?;
        if progression.game_mode != GameMode::Expedition
            || pending.depth != progression.floor_index().saturating_add(1)
            || !matches!(phase, GameState::NodeConfirm { node_id, kind_id, category }
                if *node_id == pending.node_id && *kind_id == node.kind_id && *category == node.category)
            || session != Some(&crate::game::map::NodeSession::from(node))
            || node.state != crate::game::map::MapNodeState::Available
            || !navigation.is_node_selectable(map, &template, pending.node_id) {
            return Err(GameError::InvalidAction);
        }
        match pending.cause {
            Cause::Gate if node.category == crate::game::map::MapNodeCategory::Gate
                && node.payload == crate::game::map::MapNodePayload::None && pending.depth >= 3 => Ok(()),
            Cause::Exhaustion if !matches!(node.category, crate::game::map::MapNodeCategory::Gate | crate::game::map::MapNodeCategory::Start)
                && progression.deterrence == 0 && progression.entry_receipts.contains(&pending.node_id) => Ok(()),
            _ => Err(GameError::InvalidAction),
        }
    }

    pub(super) fn regulated_node_entry_cost(
        &self,
        node: &crate::game::map::MapNode,
        facility: &crate::game::map::FacilityRunState,
    ) -> Result<u32, GameError> {
        use crate::game::map::MapNodeCategory;
        if matches!(node.category, MapNodeCategory::Start | MapNodeCategory::Gate) {
            return Ok(0);
        }
        if let Some(rooms) = &facility.rooms {
            let layout = self.game_data.map_catalogs.facility_templates.layouts.iter()
                .find(|layout| layout.id == rooms.layout_id)
                .ok_or(GameError::InvalidAction)?;
            if let Some(cost) = rooms.activity_metadata.get(&node.id)
                .and_then(|meta| layout.activity(&meta.activity_id))
                .and_then(|activity| activity.first_entry_cost) {
                return Ok(cost);
            }
        }
        let authored_cost = self.game_data.map_catalogs.map_node_definitions.nodes.iter()
            .find(|definition| definition.kind_id == node.kind_id)
            .and_then(|definition| definition.first_entry_cost);
        // Published kinds are checked by the loader. Internal development nodes
        // without a kind override inherit the same authored run default.
        Ok(authored_cost.unwrap_or(self.run_policy().expedition_first_entry_cost))
    }

    pub(super) fn node_entry_preview(
        &self,
        node: &crate::game::map::MapNode,
    ) -> Result<crate::game::map::NodeEntryPreviewDto, GameError> {
        let room_id = self.run_state()?.facility_run_state.rooms.as_ref()
            .map(|rooms| rooms.current_room_id.as_str());
        self.node_entry_preview_in_room(node, room_id)
    }

    pub(super) fn node_entry_preview_in_room(
        &self,
        node: &crate::game::map::MapNode,
        room_id: Option<&str>,
    ) -> Result<crate::game::map::NodeEntryPreviewDto, GameError> {
        let run = self.run_state()?;
        let sealed = room_id.and_then(|id| self.sealed_golden_bough_in_room(id, node)).is_some();
        let expedition = run.run_progression.game_mode == GameMode::Expedition;
        let regulated_cost = if expedition {
            self.regulated_node_entry_cost(node, &run.facility_run_state)?
        } else { 0 };
        let charged_cost = if matches!(node.state, crate::game::map::MapNodeState::Completed)
            || run.run_progression.entry_receipts.contains(&node.id) { 0 } else { regulated_cost };
        let deterrence_after = run.run_progression.deterrence.saturating_sub(charged_cost);
        let mut breakdown: Vec<_> = self.state.roster.iter()
            .filter(|employee| employee.is_alive() && employee.condition.is_breakdown())
            .map(|employee| {
                let remaining = employee.breakdown.remaining_cost()
                    .unwrap_or(self.run_policy().corrosion.breakdown_collapse_cost_limit);
                crate::game::map::BreakdownEntryPreviewDto {
                    employee_uuid: employee.uuid,
                    remaining_cost_before: remaining,
                    remaining_cost_after: remaining.saturating_sub(charged_cost),
                    dies_on_entry: charged_cost > 0 && charged_cost >= remaining,
                }
            }).collect();
        breakdown.sort_by_key(|entry| entry.employee_uuid);
        let stress_policy = (self.basic_relic_activity(node.id) == Some(crate::game::map::RewardActivityKind::StressCombat))
            .then(|| self.facility_layout().and_then(|l| l.stress_combat.as_ref())).flatten();
        let mut combat_deterrence = match &node.payload {
            crate::game::map::MapNodePayload::Encounter { encounter_id: Some(id) } => self.game_data.pve_data.get_by_id(id)
                .map(|encounter| crate::game::combat_preview::BattleDeterrenceSnapshot::for_entry(
                    encounter.deterrence_tuning, expedition && deterrence_after == 0, sealed)),
            _ => None,
        };
        let stress_combat = stress_policy.map(|policy| {
            let encounter = self.game_data.pve_data.get_by_id(&policy.encounter_id).ok_or(GameError::InvalidAction)?;
            let snapshot = crate::game::combat_preview::BattleDeterrenceSnapshot::for_entry(encounter.deterrence_tuning, expedition && deterrence_after == 0, false);
            combat_deterrence = Some(snapshot);
            Ok::<_, GameError>(crate::game::stress_combat::StressCombatEntryView { total_rounds: 5, quantity_multiplier: snapshot.victory_quantity_multiplier, first_rewards: policy.rewards_through(1, snapshot.victory_quantity_multiplier)? })
        }).transpose()?;
        Ok(crate::game::map::NodeEntryPreviewDto {
            stress_combat,
            combat_deterrence,
            sealed_golden_bough: sealed,
            regulated_cost, charged_cost, deterrence_after,
            unrestrained_after_entry: expedition && deterrence_after == 0,
            breakdown,
        })
    }

    pub(super) fn prepare_expedition_node_entry(&mut self, node_id: crate::game::map::MapNodeId) -> Result<bool, GameError> {
        use crate::game::expedition_lifecycle::{ExpeditionContinuationCause as Cause, PendingExpeditionContinuation};
        let run = self.run_state()?;
        if run.run_progression.game_mode != GameMode::Expedition { return Ok(false); }
        if run.run_progression.pending_continuation.is_some() { return Err(GameError::InvalidAction); }
        let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
        let category = node.category;
        let first = node.state == crate::game::map::MapNodeState::Available
            && !run.run_progression.entry_receipts.contains(&node_id);
        let depth = run.run_progression.floor_index().saturating_add(1);
        let cost = self.regulated_node_entry_cost(node, &run.facility_run_state)?;
        let progression = &mut self.run_state_mut()?.run_progression;
        let cause = if category == crate::game::map::MapNodeCategory::Gate {
            (depth >= 3).then_some(Cause::Gate)
        } else if first && category != crate::game::map::MapNodeCategory::Start {
            progression.entry_receipts.insert(node_id);
            let before = progression.deterrence;
            progression.deterrence = before.saturating_sub(cost);
            (before > 0 && progression.deterrence == 0).then_some(Cause::Exhaustion)
        } else { None };
        if first && !matches!(category, crate::game::map::MapNodeCategory::Start | crate::game::map::MapNodeCategory::Gate) {
            self.consume_employee_breakdown_entry_cost(cost)?;
        }
        if let Some(cause) = cause {
            let progression = &mut self.run_state_mut()?.run_progression;
            progression.pending_continuation = Some(PendingExpeditionContinuation { node_id, depth, cause });
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn handle_choose_expedition_continuation(&mut self, choice: crate::game::expedition_lifecycle::ExpeditionContinuationChoice) -> Result<BehaviorResult, GameError> {
        use crate::game::expedition_lifecycle::{ExpeditionContinuationCause as Cause, ExpeditionContinuationChoice as Choice};
        if choice == Choice::Return && self.facility_gate_reached() && self.run_state()?.run_progression.floor_index() >= 2 {
            let map = self.current_map_view()?;
            self.settle_expedition_terminal(true)?;
            self.state.node_session = None;
            self.state.active_node_content = None;
            self.transition_to(GameState::RunComplete)?;
            self.record_facility_room_completion();
            return Ok(BehaviorResult::RunComplete {map});
        }
        let run = self.run_state()?;
        let pending = run.run_progression.pending_continuation.ok_or(GameError::InvalidAction)?;
        if run.run_progression.game_mode != GameMode::Expedition || pending.depth != run.run_progression.floor_index().saturating_add(1) {
            return Err(GameError::InvalidAction);
        }
        self.validate_pending_expedition_selection(&run.run_progression, &run.map, &run.map_progression,
            self.state.lifecycle.phase(), self.state.node_session.as_ref())?;
        match choice {
            Choice::Return => {
                if pending.depth < 3 { return Err(GameError::InvalidAction); }
                let map = self.current_map_view()?;
                self.settle_expedition_terminal(true)?;
                self.state.node_session = None;
                self.state.active_node_content = None;
                self.transition_to(GameState::RunComplete)?;
                if pending.cause == Cause::Gate { self.record_facility_room_completion(); }
                Ok(BehaviorResult::RunComplete { map })
            }
            Choice::Abandon => {
                if pending.cause != Cause::Exhaustion || pending.depth >= 3 { return Err(GameError::InvalidAction); }
                self.handle_abandon_run()
            }
            Choice::Continue => {
                let recovery_restricted = self.has_combat_recovery_restriction();
                if recovery_restricted && pending.cause == Cause::Exhaustion
                    && !self.recovery_node_is_useful(self.current_recovery_context()?, pending.node_id, false)
                {
                    return Err(GameError::InvalidAction);
                }
                self.run_state_mut()?.run_progression.pending_continuation = None;
                self.state.node_session = None;
                self.transition_to(GameState::ViewingMap)?;
                self.handle_select_map_node(pending.node_id)?;
                let result = self.handle_confirm_enter_node_resuming(true)?;
                if recovery_restricted && pending.cause != Cause::Gate
                    && self.reassess_combat_recovery()?.is_some()
                {
                    // The public application draft discards this attempted continuation before publication.
                    return Err(GameError::InvalidAction);
                }
                Ok(result)
            }
        }
    }

    pub(super) fn handle_skip_tutorial(&mut self) -> Result<BehaviorResult, GameError> {
        if self.state.pending_game_mode != Some(GameMode::Tutorial)
            || !matches!(self.state.lifecycle.phase(), GameState::PreRunBriefing { game_mode: GameMode::Tutorial }) {
            return Err(GameError::InvalidAction);
        }
        let player = self.state.player.clone();
        let profile = self.capture_player_profile()?;
        self.state = GameCoreState::new(self.run_seed);
        self.state.player = player;
        self.state.player_profile = profile;
        Ok(BehaviorResult::RunAbandoned)
    }

    pub(super) fn handle_acknowledge_first_return_story(&mut self) -> Result<BehaviorResult, GameError> {
        if self.state.pending_game_mode == Some(GameMode::Tutorial)
            || self.state.run.as_ref().is_some_and(|run| run.run_progression.game_mode == GameMode::Tutorial) {
            return Err(GameError::InvalidAction);
        }
        match self.state.player_profile.lifecycle.first_return_story {
            FirstReturnStory::Never => return Err(GameError::InvalidAction),
            FirstReturnStory::Pending => self.state.player_profile.lifecycle.first_return_story = FirstReturnStory::Consumed,
            FirstReturnStory::Consumed => {},
        }
        Ok(BehaviorResult::Ok)
    }
}
