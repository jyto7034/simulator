use rand::{Rng, SeedableRng};
use uuid::Uuid;
use super::{GameCore, grant_state::EconomyGrantState};
use crate::game::{
    behavior::{BehaviorResult, GameError},
    data::relic_data::RelicDefinition,
    determinism,
    enums::RewardMode,
    map::{MapNodeId, RewardActivityKind},
    relic_reward::*,
    resources::{GameState, RewardSessionState},
    reward::{GrantExecutionContext, RewardEffect, RewardOption},
};
use RelicRewardBlockedReason as Blocked;

const CHOICE_SEED: u64 = 0x5531_3632_4348_4F49;
const INVEST_SEED: u64 = 0x5531_3632_4455_5354;
const FUSION_SEED: u64 = 0x5531_3632_4655_5345;

impl GameCore {
    pub(super) fn basic_relic_activity(&self, node_id: MapNodeId) -> Option<RewardActivityKind> {
        self.state.run.as_ref()?.facility_run_state.rooms.as_ref()?.activity_metadata.get(&node_id)?.reward_activity
    }
    pub(super) fn basic_relic_activity_supported(kind: RewardActivityKind) -> bool {
        matches!(kind, RewardActivityKind::SimpleChoice | RewardActivityKind::ResourceInvestment | RewardActivityKind::RelicGacha)
    }
    fn relic_reward_policy(&self) -> Result<&BasicRelicRewardPolicy, GameError> {
        Ok(&self.facility_layout().ok_or(GameError::InvalidAction)?.relic_rewards)
    }
    fn relic_activity_session(&self) -> Result<&RewardSessionState, GameError> {
        if !matches!(self.state.lifecycle.phase(), GameState::InReward { .. }) { return Err(GameError::NotInRewardState); }
        let reward = self.state.active_node_content.as_ref().ok_or(GameError::NotInRewardState)?.as_reward()?;
        if reward.relic_activity.is_none() { return Err(GameError::InvalidAction); }
        Ok(reward)
    }
    /// Resolve the authored pool once per operation. B-tree lookup avoids an
    /// unmetered catalog scan for every candidate in expanded content packs.
    fn relic_reward_candidates(&self, grade: Option<RelicRewardGrade>, excluded: [&str; 2]) -> Result<Vec<(&RelicDefinition, u32)>, GameError> {
        let policy = self.relic_reward_policy()?;
        let database = &self.game_data.relic_data;
        let count = database.relics.len().checked_add(policy.candidates.len()).ok_or(GameError::InvalidAction)?;
        crate::authoring::control::charge(count as u64, "relic_reward_candidate_scan")?;
        let bytes = database.relics.len().checked_mul(11 * std::mem::size_of::<(&str, &RelicDefinition)>() + 224)
            .and_then(|n| n.checked_add(policy.candidates.len().checked_mul(std::mem::size_of::<(&RelicDefinition, u32)>())?));
        crate::game::battle::runtime_control::admit_retained_growth(0, bytes, "relic_reward_candidates")?;
        let index: std::collections::BTreeMap<_, _> = database.relics.iter().map(|d| (d.id.as_str(), d)).collect();
        let mut candidates = Vec::with_capacity(policy.candidates.len());
        for entry in &policy.candidates {
            let definition = *index.get(entry.relic_id.as_str()).ok_or(GameError::InvalidAction)?;
            if excluded.contains(&definition.id.as_str()) { continue; }
            if grade.is_some_and(|g| RelicRewardGrade::from_rarity(&definition.rarity) != Some(g)) { continue; }
            candidates.push((definition, entry.weight));
        }
        Ok(candidates)
    }
    fn sample_relic_rewards(&self, grade: Option<RelicRewardGrade>, excluded: [&str; 2], seed: u64, count: usize) -> Result<Vec<RewardOption>, GameError> {
        let mut candidates = self.relic_reward_candidates(grade, excluded)?;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let draws = count.min(candidates.len());
        crate::game::battle::runtime_control::admit_retained_growth(0, draws.checked_mul(std::mem::size_of::<RewardOption>()), "relic_reward_option_vec")?;
        let mut rewards = Vec::with_capacity(draws);
        for draw in 0..count.min(candidates.len()) {
            crate::authoring::control::charge(candidates.len() as u64, "relic_reward_weighted_draw")?;
            let total = candidates.iter().try_fold(0u32, |n, (_,w)| n.checked_add(*w)).ok_or(GameError::InvalidAction)?;
            if total == 0 { return Err(GameError::InvalidAction); }
            let mut roll = rng.gen_range(0..total);
            let selected = candidates.iter().position(|(_,weight)| {
                if roll < *weight { true } else { roll -= *weight; false }
            }).ok_or(GameError::InvalidAction)?;
            let (definition, _) = candidates.remove(selected);
            let bytes = definition.id.len().checked_mul(2)
                .and_then(|n| n.checked_add(definition.name.len()))
                .and_then(|n| n.checked_add(definition.description.len()))
                .and_then(|n| n.checked_add(std::mem::size_of::<RewardEffect>()));
            crate::game::battle::runtime_control::admit_retained_growth(0, bytes, "relic_reward_option_strings")?;
            let option = RewardOption {
                id: definition.id.clone(),
                uuid: determinism::uuid_v4_from_seed(seed, CHOICE_SEED, draw as u64),
                name: definition.name.clone(), description: definition.description.clone(), icon: String::new(),
                effects: vec![RewardEffect::GrantRelic { relic_id: definition.id.clone() }],
            };
            crate::authoring::control::charge_serialized(&option, "relic_reward_option")?;
            rewards.push(option);
        }
        Ok(rewards)
    }
    pub(super) fn resolve_basic_relic_reward(&self, node_id: MapNodeId, kind: RewardActivityKind) -> Result<RewardSessionState, GameError> {
        if !Self::basic_relic_activity_supported(kind) { return Err(GameError::NotImplemented("relic activity".into())); }
        self.ensure_relic_operation()?;
        let rewards = if kind == RewardActivityKind::SimpleChoice {
            self.sample_relic_rewards(None, ["", ""], self.node_seed(node_id, CHOICE_SEED), 3)?
        } else { vec![] };
        let mut reward = self.build_reward_session(node_id.0, RewardMode::ChooseOne, rewards, true);
        reward.relic_activity = Some(RelicRewardSession { kind, investment_grade: None, paid_dust: 0, fusion_inputs: None });
        Ok(reward)
    }
    pub(super) fn relic_investment_preview(&self, activity: &RelicRewardSession) -> Result<RelicInvestmentPreview, GameError> {
        let next_grade = RelicRewardGrade::next(activity.investment_grade);
        let additional_dust = next_grade.map(|g| self.relic_reward_policy().map(|p| p.dust_thresholds[g.index()]))
            .transpose()?.map(|n| n.checked_sub(activity.paid_dust).ok_or(GameError::InvalidAction)).transpose()?;
        let reason = match next_grade {
            None => Some(Blocked::MaximumGrade),
            Some(grade) if self.relic_reward_candidates(Some(grade), ["", ""])?.is_empty() => Some(Blocked::NoCandidates),
            Some(_) if self.state.crafting_resources.equipment_dust < additional_dust.unwrap_or(0) => Some(Blocked::InsufficientDust),
            Some(_) => None,
        };
        Ok(RelicInvestmentPreview { next_grade, additional_dust, eligibility: RelicRewardEligibility::new(reason) })
    }
    fn fusion_input_grade(&self, copy: Uuid) -> Result<Result<RelicRewardGrade, Blocked>, GameError> {
        let Some(owned) = self.state.relics.get(copy) else { return Ok(Err(Blocked::MissingCopy)); };
        if self.state.relics.equipped_to(copy)?.is_some() { return Ok(Err(Blocked::EquippedCopy)); }
        crate::authoring::control::charge(self.game_data.relic_data.relics.len() as u64, "relic_fusion_input_definition")?;
        Ok(self.game_data.relic_data.get_by_id(&owned.definition_id)
            .and_then(|d| RelicRewardGrade::from_rarity(&d.rarity)).ok_or(Blocked::UnsupportedGrade))
    }
    pub(super) fn relic_fusion_preview(&self, inputs: Option<[Uuid; 2]>) -> Result<RelicFusionPreview, GameError> {
        let mut view = RelicFusionPreview { inputs, probabilities: None, eligibility: RelicRewardEligibility::new(Some(Blocked::SelectTwoCopies)) };
        let Some([a,b]) = inputs else { return Ok(view); };
        if a == b { view.eligibility = RelicRewardEligibility::new(Some(Blocked::SameCopy)); return Ok(view); }
        let ga = self.fusion_input_grade(a)?;
        let gb = self.fusion_input_grade(b)?;
        let (ga,gb) = match (ga,gb) {
            (Ok(ga),Ok(gb)) => (ga,gb),
            (Err(reason),_) | (_,Err(reason)) => { view.eligibility = RelicRewardEligibility::new(Some(reason)); return Ok(view); }
        };
        let chances = fusion_probabilities(ga,gb);
        view.probabilities = Some(chances);
        let excluded = [self.state.relics.get(a).unwrap().definition_id.as_str(), self.state.relics.get(b).unwrap().definition_id.as_str()];
        let mut present = [false;3];
        for (definition,_) in self.relic_reward_candidates(None, excluded)? {
            let grade = RelicRewardGrade::from_rarity(&definition.rarity).ok_or(GameError::InvalidAction)?;
            present[grade.index()] = true;
        }
        view.eligibility = RelicRewardEligibility::new((0..3).any(|i| chances[i] > 0 && !present[i]).then_some(Blocked::MissingOutcomeGrade));
        Ok(view)
    }
    pub(super) fn handle_invest_relic_reward(&mut self) -> Result<BehaviorResult, GameError> {
        self.ensure_relic_operation()?;
        let current = self.relic_activity_session()?;
        let activity = current.relic_activity.as_ref().unwrap();
        if activity.kind != RewardActivityKind::ResourceInvestment { return Err(GameError::InvalidAction); }
        let preview = self.relic_investment_preview(activity)?;
        if !preview.eligibility.enabled { return Err(GameError::InvalidAction); }
        let grade = preview.next_grade.ok_or(GameError::InvalidAction)?;
        let cost = preview.additional_dust.ok_or(GameError::InvalidAction)?;
        let seed = determinism::seed_with_namespace(self.node_seed(MapNodeId(current.stage_uuid), INVEST_SEED), grade.index() as u64);
        let rewards = self.sample_relic_rewards(Some(grade), ["", ""], seed, 3)?;
        if rewards.is_empty() { return Err(GameError::InvalidAction); }
        // All fallible preparation precedes the single in-memory commit.
        let remaining = self.state.crafting_resources.equipment_dust.checked_sub(cost).ok_or(GameError::InvalidAction)?;
        let paid = activity.paid_dust.checked_add(cost).ok_or(GameError::InvalidAction)?;
        current.preflight_relic_activity_clone()?;
        let mut next = crate::authoring::clone_shape::controlled_clone(current)?;
        next.rewards = rewards; next.selected_reward_uuid = None;
        let activity = next.relic_activity.as_mut().unwrap();
        activity.investment_grade = Some(grade); activity.paid_dust = paid;
        let result = self.reward_state_result(&next);
        self.state.crafting_resources.equipment_dust = remaining;
        self.state.active_node_content = Some(crate::game::resources::ActiveNodeContent::Reward(next));
        Ok(result)
    }
    pub(super) fn handle_select_relic_fusion_inputs(&mut self, mut copies: Option<[Uuid;2]>) -> Result<BehaviorResult, GameError> {
        self.ensure_relic_operation()?;
        let current = self.relic_activity_session()?;
        if current.relic_activity.as_ref().unwrap().kind != RewardActivityKind::RelicGacha { return Err(GameError::InvalidAction); }
        if let Some(inputs) = &mut copies {
            inputs.sort();
            if inputs[0] == inputs[1] || self.fusion_input_grade(inputs[0])?.is_err() || self.fusion_input_grade(inputs[1])?.is_err() { return Err(GameError::InvalidAction); }
        }
        current.preflight_relic_activity_clone()?;
        let mut next = crate::authoring::clone_shape::controlled_clone(current)?;
        next.relic_activity.as_mut().unwrap().fusion_inputs = copies;
        let result = self.reward_state_result(&next);
        self.state.active_node_content = Some(crate::game::resources::ActiveNodeContent::Reward(next));
        Ok(result)
    }
    pub(super) fn handle_confirm_relic_fusion(&mut self) -> Result<BehaviorResult, GameError> {
        self.ensure_relic_operation()?;
        let current = self.relic_activity_session()?;
        let activity = current.relic_activity.as_ref().unwrap();
        if activity.kind != RewardActivityKind::RelicGacha { return Err(GameError::InvalidAction); }
        let preview = self.relic_fusion_preview(activity.fusion_inputs)?;
        if !preview.eligibility.enabled { return Err(GameError::InvalidAction); }
        let mut inputs = activity.fusion_inputs.ok_or(GameError::InvalidAction)?; inputs.sort();
        let seed = inputs.iter().enumerate().fold(self.node_seed(MapNodeId(current.stage_uuid), FUSION_SEED), |seed,(i,id)| determinism::seed_with_uuid(seed,i as u64,*id));
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let roll = rng.gen_range(0..100);
        let chances = preview.probabilities.ok_or(GameError::InvalidAction)?;
        let grade = if roll < chances[0] { RelicRewardGrade::Common } else if roll < chances[0]+chances[1] { RelicRewardGrade::Rare } else { RelicRewardGrade::Special };
        let excluded = [self.state.relics.get(inputs[0]).unwrap().definition_id.as_str(),self.state.relics.get(inputs[1]).unwrap().definition_id.as_str()];
        let option = self.sample_relic_rewards(Some(grade), excluded, determinism::seed_with_namespace(seed,1), 1)?.pop().ok_or(GameError::InvalidAction)?;
        self.finish_basic_relic_reward(option, inputs.to_vec())
    }
    pub(super) fn claim_basic_relic_reward(&mut self) -> Result<BehaviorResult, GameError> {
        self.ensure_relic_operation()?;
        let current = self.relic_activity_session()?;
        if current.relic_activity.as_ref().unwrap().kind == RewardActivityKind::RelicGacha { return Err(GameError::InvalidAction); }
        let option = crate::authoring::clone_shape::controlled_clone(current.get_selected_reward().ok_or(GameError::InvalidAction)?)?;
        self.finish_basic_relic_reward(option, vec![])
    }
    fn finish_basic_relic_reward(&mut self, option: RewardOption, consumed: Vec<Uuid>) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(|core| {
            let current = core.relic_activity_session()?;
            let activity = current.relic_activity.as_ref().unwrap();
            let (node_id, kind, paid_dust) = (MapNodeId(current.stage_uuid), activity.kind, activity.paid_dust);
            let mut assets = EconomyGrantState::capture(&core.state)?;
            for id in &consumed { assets.relics.remove(*id)?; }
            let result = assets.grant_effects(&core.game_data, &core.state.skill_fragment_policy, &GrantExecutionContext::default(), &option.effects, core.node_seed(node_id, FUSION_SEED))?;
            let receipt = RelicRewardReceipt { node_id, kind, granted: result.relic_diffs, consumed_copies: consumed, paid_dust, equipment_dust: assets.crafting_resources.equipment_dust };
            assets.commit_into(&mut core.state);
            core.handle_complete_node()?;
            Ok(BehaviorResult::RewardGranted { enkephalin: core.state.enkephalin.amount, inventory_diff: result.inventory_diff, skill_fragment_diffs: result.skill_fragment_diffs, employee_experience_diffs: result.employee_experience_diffs, relic_activity: Some(receipt) })
        })
    }
    pub(super) fn relic_reward_view(&self, reward: &RewardSessionState) -> Result<Option<RelicRewardView>, GameError> {
        let Some(activity) = &reward.relic_activity else { return Ok(None); };
        if !crate::game::battle::runtime_control::has_operation_scope() {
            return crate::game::battle::runtime_control::run_deferred(|| self.relic_reward_view(reward));
        }
        self.ensure_relic_operation()?;
        crate::game::battle::runtime_control::admit_retained_growth(0, reward.rewards.len().checked_mul(std::mem::size_of::<RelicDefinition>()), "relic_reward_candidate_view_vec")?;
        let mut candidates = Vec::with_capacity(reward.rewards.len());
        for option in &reward.rewards {
            crate::authoring::control::charge(self.game_data.relic_data.relics.len() as u64, "relic_reward_candidate_view_lookup")?;
            let definition = self.game_data.relic_data.get_by_id(&option.id).ok_or(GameError::InvalidAction)?;
            crate::game::relic::allocation::admit_owned_projection(definition, "relic_reward_candidate_view")?;
            candidates.push(crate::authoring::clone_shape::controlled_clone(definition)?);
        }
        let investment = (activity.kind == RewardActivityKind::ResourceInvestment).then(|| self.relic_investment_preview(activity)).transpose()?;
        let fusion = (activity.kind == RewardActivityKind::RelicGacha).then(|| self.relic_fusion_preview(activity.fusion_inputs)).transpose()?;
        let mut fusion_inputs = vec![];
        if fusion.is_some() {
            crate::game::battle::runtime_control::admit_retained_growth(0, self.state.relics.copy_count().checked_mul(std::mem::size_of::<RelicFusionInputView>()), "relic_fusion_input_view")?;
            fusion_inputs.reserve_exact(self.state.relics.copy_count());
            for copy in self.state.relics.copies() {
                let result = self.fusion_input_grade(copy.copy_uuid)?;
                crate::authoring::control::charge(self.game_data.relic_data.relics.len() as u64, "relic_fusion_input_view_grade")?;
                crate::game::relic::allocation::admit_owned_projection(copy, "relic_fusion_copy_view")?;
                fusion_inputs.push(RelicFusionInputView { copy: crate::authoring::clone_shape::controlled_clone(copy)?, grade: self.game_data.relic_data.get_by_id(&copy.definition_id).and_then(|d| RelicRewardGrade::from_rarity(&d.rarity)), eligibility: RelicRewardEligibility::new(result.err()) });
            }
        }
        Ok(Some(RelicRewardView { kind: activity.kind, investment_grade: activity.investment_grade, paid_dust: activity.paid_dust, equipment_dust: self.state.crafting_resources.equipment_dust, candidates, no_candidates_reason: (reward.rewards.is_empty() && activity.kind == RewardActivityKind::SimpleChoice).then_some(Blocked::NoCandidates), investment, fusion, fusion_inputs }))
    }
    pub(super) fn relic_reward_entry_view(&self, kind: RewardActivityKind) -> Option<RelicRewardEntryView> {
        if !Self::basic_relic_activity_supported(kind) { return None; }
        Some(RelicRewardEntryView { kind, dust_thresholds: (kind == RewardActivityKind::ResourceInvestment).then(|| self.facility_layout().map(|l| l.relic_rewards.dust_thresholds)).flatten(), requires_two_distinct_unequipped_copies: kind == RewardActivityKind::RelicGacha, corrupt_inputs_allowed: kind == RewardActivityKind::RelicGacha })
    }
}
