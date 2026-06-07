use super::{GameCore, RUN_SYSTEM_POLICY};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::employee::Employee;
use crate::game::enums::ShopEventOption;
use crate::game::map::{MapNodePayload, NodeSessionKind};
use crate::game::resources::{ActiveNodeContent, GameState, InventoryDiffDto};

impl GameCore {
    fn current_headquarters_contact(
        &self,
    ) -> Result<crate::game::resources::HeadquartersContactSessionState, GameError> {
        let Some(node_session) = self.state.node_session.as_ref() else {
            return Err(GameError::InvalidAction);
        };
        if node_session.session_kind != NodeSessionKind::HeadquartersContact {
            return Err(GameError::InvalidAction);
        }
        let selected = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        Ok(selected.as_headquarters_contact()?.clone())
    }

    fn complete_headquarters_contact_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.state.active_node_content = None;
        self.handle_complete_node()
    }

    pub(super) fn handle_recruit_employee(
        &mut self,
        candidate_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        let headquarters = self.current_headquarters_contact()?;
        let candidate = headquarters
            .get_candidate(candidate_id)
            .ok_or(GameError::InvalidAction)?
            .clone();

        let employee_uuid = self.state.uuid_manager.next_employee();
        self.roster_mut()?
            .add(Employee::from_starter_candidate(employee_uuid, &candidate));
        self.sync_roster_order_with_owned_units()?;

        let completion = self.complete_headquarters_contact_node()?;
        Ok(BehaviorResult::EmployeeRecruited {
            candidate_id: candidate.id,
            employee_uuid,
            completion: Box::new(completion),
        })
    }

    pub(super) fn handle_request_emergency_supplies(
        &mut self,
    ) -> Result<BehaviorResult, GameError> {
        let _headquarters = self.current_headquarters_contact()?;
        self.state.enkephalin.amount = self
            .state
            .enkephalin
            .amount
            .saturating_add(RUN_SYSTEM_POLICY.headquarters.emergency_enkephalin);
        let enkephalin = self.state.enkephalin.amount;
        let inventory_diff = InventoryDiffDto::default();

        let completion = self.complete_headquarters_contact_node()?;
        Ok(BehaviorResult::EmergencySuppliesGranted {
            enkephalin,
            inventory_diff,
            completion: Box::new(completion),
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
        let shop_result = ShopEventOption {
            id: shop.id.clone(),
            name: shop.name.clone(),
            uuid: shop.uuid,
            shop_type: shop.shop_type,
            can_reroll: shop.can_reroll,
            visible_items: shop.visible_items.clone(),
        };
        self.state.active_node_content = Some(ActiveNodeContent::Shop(shop));
        self.transition_to(GameState::InShop { shop_uuid })?;
        Ok(BehaviorResult::ShopState {
            shop: shop_result,
            research_deliveries: vec![],
        })
    }
}
