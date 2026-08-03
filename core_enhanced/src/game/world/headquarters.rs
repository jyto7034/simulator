use super::node_flow::StagedNodeCompletion;
use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::employee::Employee;
use crate::game::map::{MapNodeCategory, MapNodePayload};
use crate::game::resources::{ActiveNodeContent, GameState};

impl GameCore {
    fn current_headquarters_contact(
        &self,
    ) -> Result<crate::game::resources::HeadquartersContactSessionState, GameError> {
        let Some(node_session) = self.state.node_session.as_ref() else {
            return Err(GameError::InvalidAction);
        };
        if node_session.category != MapNodeCategory::HeadquartersContact {
            return Err(GameError::InvalidAction);
        }
        let selected = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        Ok(selected.as_headquarters_contact()?.clone())
    }

    fn plan_headquarters_contact_completion(&mut self) -> Result<StagedNodeCompletion, GameError> {
        let active_content = self.state.active_node_content.take();
        let completion = self.plan_complete_current_node(true);
        self.state.active_node_content = active_content;
        completion
    }

    fn with_headquarters_transaction<T>(
        &mut self,
        apply: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        let rollback_game_state = self.state.game_state.clone();
        let rollback_active_content = self.state.active_node_content.clone();
        let rollback_node_session = self.state.node_session.clone();
        let rollback_run = self.state.run.clone();
        let rollback_uuid_manager = self.state.uuid_manager.clone();
        let rollback_enkephalin = self.state.enkephalin.clone();
        let rollback_inventory = self.state.inventory.clone();
        let rollback_roster = self.state.roster.clone();
        let rollback_skill_fragments = self.state.skill_fragments.clone();
        let rollback_checkpoint = self.state.run_checkpoint.clone();

        match apply(self) {
            Ok(result) => Ok(result),
            Err(error) => {
                self.state.game_state = rollback_game_state;
                self.state.active_node_content = rollback_active_content;
                self.state.node_session = rollback_node_session;
                self.state.run = rollback_run;
                self.state.uuid_manager = rollback_uuid_manager;
                self.state.enkephalin = rollback_enkephalin;
                self.state.inventory = rollback_inventory;
                self.state.roster = rollback_roster;
                self.state.skill_fragments = rollback_skill_fragments;
                self.state.run_checkpoint = rollback_checkpoint;
                Err(error)
            }
        }
    }

    pub(super) fn handle_recruit_employee(
        &mut self,
        employee_definition_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        let headquarters = self.current_headquarters_contact()?;
        let active_session = self
            .run_state()?
            .recruitment
            .active_session()
            .ok_or(GameError::InvalidAction)?;
        if active_session.node_id != headquarters.node_id
            || !active_session
                .employee_definition_ids
                .iter()
                .any(|id| id == employee_definition_id)
        {
            return Err(GameError::InvalidAction);
        }
        let candidate = self
            .game_data
            .employee_definition_data
            .get_by_id(employee_definition_id)
            .cloned()
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "recruitment session references missing employee definition '{}'",
                    employee_definition_id
                ))
            })?;
        let mut recruitment = self.run_state()?.recruitment.clone();
        recruitment.recruit_from_active_session(employee_definition_id)?;
        let mut uuid_manager = self.state.uuid_manager.clone();
        let employee_uuid = uuid_manager.next_employee();
        let mut roster = self.state.roster.clone();
        roster.add(Employee::from_definition(employee_uuid, &candidate));
        recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            &roster,
        )?;
        let staged_completion = self.plan_headquarters_contact_completion()?;

        self.with_headquarters_transaction(move |core| {
            core.run_state_mut()?.recruitment = recruitment;
            core.state.uuid_manager = uuid_manager;
            core.state.roster = roster;
            core.state.active_node_content = None;
            let completion = core.commit_interactive_node_completion(staged_completion)?;
            Ok(BehaviorResult::EmployeeRecruited {
                employee_definition_id: candidate.employee_definition_id,
                employee_uuid,
                completion: Box::new(completion),
            })
        })
    }

    pub(super) fn handle_request_emergency_supplies(
        &mut self,
    ) -> Result<BehaviorResult, GameError> {
        let _headquarters = self.current_headquarters_contact()?;
        let mut recruitment = self.run_state()?.recruitment.clone();
        recruitment.return_active_session_to_tail()?;
        recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            &self.state.roster,
        )?;
        let emergency_enkephalin = self.run_policy().headquarters.emergency_enkephalin;
        let mut enkephalin = self.state.enkephalin.clone();
        let enkephalin_amount = enkephalin.checked_add(emergency_enkephalin)?;
        let staged_completion = self.plan_headquarters_contact_completion()?;

        self.with_headquarters_transaction(move |core| {
            core.run_state_mut()?.recruitment = recruitment;
            core.state.enkephalin = enkephalin;
            core.state.active_node_content = None;
            let completion = core.commit_interactive_node_completion(staged_completion)?;
            Ok(BehaviorResult::EmergencySuppliesGranted {
                enkephalin: enkephalin_amount,
                inventory_diff: Default::default(),
                completion: Box::new(completion),
            })
        })
    }

    pub(super) fn handle_open_headquarters_shop(&mut self) -> Result<BehaviorResult, GameError> {
        let headquarters = self.current_headquarters_contact()?;
        let shop_payload = MapNodePayload::Shop {
            shop_id: None,
            shop_pool_id: headquarters.shop_pool_id.clone(),
        };
        let Some(shop) = self.resolve_map_shop(headquarters.node_id, &shop_payload)? else {
            return Err(GameError::InvalidStaticData(
                "headquarters contact node could not resolve headquarters shop".to_string(),
            ));
        };

        let shop_uuid = shop.uuid;
        let shop_result = Self::shop_event_option(&shop);
        let mut recruitment = self.run_state()?.recruitment.clone();
        recruitment.return_active_session_to_tail()?;
        recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            &self.state.roster,
        )?;

        self.with_headquarters_transaction(move |core| {
            let run = core.run_state_mut()?;
            run.recruitment = recruitment;
            run.shop_sessions.insert(headquarters.node_id, shop.clone());
            core.state.active_node_content = Some(ActiveNodeContent::Shop(shop));
            core.transition_to(GameState::InShop { shop_uuid })?;
            Ok(BehaviorResult::ShopState { shop: shop_result })
        })
    }
}
