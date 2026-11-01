use crate::game::{
    behavior::GameError,
    data::GameDataBase,
    employee::EmployeeRoster,
    managers::uuid_manager::UuidManager,
    resources::{Enkephalin, Inventory},
    reward::{
        GrantExecutionContext, GrantExecutionResult, GrantExecutor, RewardEffect, RewardOption,
    },
    skill_fragment::{SkillFragmentInventory, SkillFragmentPolicy},
};

use super::state::GameCoreState;

/// Owns the seven mutable roots shared by every internal grant transaction.
///
/// `GrantExecutor` remains the public ordered-effect API. This private value centralizes only the
/// Core callers' clone/apply/commit mapping; wider reward, shop, event and combat state stays with
/// those domain owners.
#[derive(Clone)]
pub(super) struct EconomyGrantState {
    pub(super) inventory: Inventory,
    pub(super) skill_fragments: SkillFragmentInventory,
    pub(super) relics: crate::game::relic::RelicInventory,
    pub(super) crafting_resources: crate::game::relic::RunCraftingResources,
    pub(super) roster: EmployeeRoster,
    pub(super) uuid_manager: UuidManager,
    pub(super) enkephalin: Enkephalin,
}

impl EconomyGrantState {
    pub(super) fn capture(state: &GameCoreState) -> Result<Self, GameError> {
        state.relics.preflight_clone()?;
        state.uuid_manager.preflight_relic_stream_clone()?;
        crate::authoring::clone_shape::capture_shape!(Self {
            inventory: &state.inventory,
            skill_fragments: &state.skill_fragments,
            relics: &state.relics,
            crafting_resources: &state.crafting_resources,
            roster: &state.roster,
            uuid_manager: &state.uuid_manager,
            enkephalin: &state.enkephalin,
        } sets {} fresh {} fresh_sets {})
    }

    pub(super) fn commit_into(self, state: &mut GameCoreState) {
        state.inventory = self.inventory;
        state.skill_fragments = self.skill_fragments;
        state.relics = self.relics;
        state.crafting_resources = self.crafting_resources;
        state.roster = self.roster;
        state.uuid_manager = self.uuid_manager;
        state.enkephalin = self.enkephalin;
    }

    pub(super) fn grant_reward(
        &mut self,
        game_data: &GameDataBase,
        skill_fragment_policy: &SkillFragmentPolicy,
        context: &GrantExecutionContext,
        reward: &RewardOption,
        seed: u64,
    ) -> Result<GrantExecutionResult, GameError> {
        GrantExecutor::grant_reward_with_run_state(
            &mut self.relics,
            &mut self.crafting_resources,
            &mut self.inventory,
            &mut self.skill_fragments,
            &mut self.roster,
            &mut self.uuid_manager,
            &mut self.enkephalin,
            game_data,
            skill_fragment_policy,
            context,
            reward,
            seed,
        )
    }

    pub(super) fn grant_effects(
        &mut self,
        game_data: &GameDataBase,
        skill_fragment_policy: &SkillFragmentPolicy,
        context: &GrantExecutionContext,
        effects: &[RewardEffect],
        seed: u64,
    ) -> Result<GrantExecutionResult, GameError> {
        GrantExecutor::grant_effects_with_run_state(
            &mut self.relics,
            &mut self.crafting_resources,
            &mut self.inventory,
            &mut self.skill_fragments,
            &mut self.roster,
            &mut self.uuid_manager,
            &mut self.enkephalin,
            game_data,
            skill_fragment_policy,
            context,
            effects,
            seed,
        )
    }
}

crate::authoring::clone_shape::struct_shape!(EconomyGrantState {
    inventory,
    skill_fragments,
    relics,
    crafting_resources,
    roster,
    uuid_manager,
    enkephalin
});
