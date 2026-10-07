use super::node_flow::{NodeCompletionContext, StagedNodeCompletion};
use super::{shop::ShopSessionCommitMode, GameCore};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::employee::Employee;
use crate::game::map::{MapNodeCategory, MapNodePayload};
use crate::game::resources::GameState;

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
        self.plan_complete_current_node_with_context(
            true,
            NodeCompletionContext::ResolvedHeadquartersContact,
        )
    }

    pub(super) fn handle_recruit_employee(
        &mut self,
        employee_definition_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        let headquarters = self.current_headquarters_contact()?;
        if self.state.roster.len() >= crate::game::employee::MAX_EMPLOYEE_ROSTER_SIZE {
            return Err(GameError::InvalidAction);
        }
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

        self.with_state_transaction(move |core| {
            core.run_state_mut()?.recruitment = recruitment;
            core.state.uuid_manager = uuid_manager;
            core.initialize_employees_with_loadouts(&[(employee_uuid, candidate.clone())])?;
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

        self.with_state_transaction(move |core| {
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

        self.with_state_transaction(move |core| {
            core.run_state_mut()?.recruitment = recruitment;
            core.commit_shop_session(shop, ShopSessionCommitMode::RequireRun)?;
            core.transition_to(GameState::InShop { shop_uuid })?;
            Ok(BehaviorResult::ShopState { shop: shop_result })
        })
    }
}
