use std::collections::{HashSet, VecDeque};

use rand::{seq::SliceRandom, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::game::{
    behavior::GameError,
    determinism,
    employee::{EmployeeDefinition, EmployeeRoster},
    map::MapNodeId,
};

const RECRUITMENT_QUEUE_SEED_NAMESPACE: u64 = 0x5245_4352_5549_5451;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecruitmentSessionState {
    pub node_id: MapNodeId,
    pub recruits_used: u8,
    pub employee_definition_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecruitmentState {
    queue: VecDeque<String>,
    active_session: Option<RecruitmentSessionState>,
}

crate::authoring::clone_shape::struct_shape!(RecruitmentState {
    queue,
    active_session
});

impl RecruitmentState {
    pub fn initialize(
        run_seed: u64,
        definitions: &[EmployeeDefinition],
        selected_definition_ids: &[String],
    ) -> Result<Self, GameError> {
        let catalog_ids = definitions
            .iter()
            .map(|definition| definition.employee_definition_id.as_str())
            .collect::<HashSet<_>>();
        let selected_ids = selected_definition_ids
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if selected_ids.len() != selected_definition_ids.len()
            || selected_ids.iter().any(|id| !catalog_ids.contains(id))
        {
            return Err(GameError::InvalidAction);
        }

        let mut queue = definitions
            .iter()
            .map(|definition| definition.employee_definition_id.clone())
            .filter(|id| !selected_ids.contains(id.as_str()))
            .collect::<Vec<_>>();
        queue.sort();
        let seed = determinism::seed_with_namespace(run_seed, RECRUITMENT_QUEUE_SEED_NAMESPACE);
        queue.shuffle(&mut rand::rngs::StdRng::seed_from_u64(seed));
        Ok(Self {
            queue: queue.into(),
            active_session: None,
        })
    }

    pub fn queue_definition_ids(&self) -> impl Iterator<Item = &str> {
        self.queue.iter().map(String::as_str)
    }

    pub fn active_session(&self) -> Option<&RecruitmentSessionState> {
        self.active_session.as_ref()
    }

    pub fn has_queued_candidates(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn begin_session(
        &mut self,
        node_id: MapNodeId,
        candidate_count: usize,
    ) -> Result<&RecruitmentSessionState, GameError> {
        if self.active_session.is_some() {
            let session = self.active_session.as_ref().expect("session exists");
            if session.node_id != node_id {
                return Err(GameError::InvalidAction);
            }
            return Ok(session);
        }

        let take_count = candidate_count.min(self.queue.len());
        let employee_definition_ids = (0..take_count)
            .filter_map(|_| self.queue.pop_front())
            .collect();
        self.active_session = Some(RecruitmentSessionState {
            node_id,
            recruits_used: 0,
            employee_definition_ids,
        });
        Ok(self.active_session.as_ref().expect("session was inserted"))
    }

    pub fn recruit_from_active_session(
        &mut self,
        employee_definition_id: &str,
    ) -> Result<(), GameError> {
        let session = self.active_session.as_mut().ok_or(GameError::InvalidAction)?;
        if session.recruits_used >= 2 { return Err(GameError::InvalidAction); }
        let index = session.employee_definition_ids.iter().position(|id| id == employee_definition_id)
            .ok_or(GameError::InvalidAction)?;
        session.employee_definition_ids.remove(index);
        session.recruits_used += 1;
        Ok(())
    }

    pub fn return_active_session_to_tail(&mut self) -> Result<(), GameError> {
        let session = self.active_session.take().ok_or(GameError::InvalidAction)?;
        self.queue.extend(session.employee_definition_ids);
        Ok(())
    }

    pub fn validate_exact_ownership(
        &self,
        definitions: &[EmployeeDefinition],
        roster: &EmployeeRoster,
    ) -> Result<(), GameError> {
        let catalog = definitions
            .iter()
            .map(|definition| definition.employee_definition_id.as_str())
            .collect::<HashSet<_>>();
        let mut owned = HashSet::with_capacity(catalog.len());
        for id in self.queue_definition_ids() {
            if !catalog.contains(id) || !owned.insert(id) {
                return Err(invalid_ownership());
            }
        }
        if let Some(session) = self.active_session() {
            if session.recruits_used > 2 { return Err(invalid_ownership()); }
            for id in &session.employee_definition_ids {
                if !catalog.contains(id.as_str()) || !owned.insert(id.as_str()) {
                    return Err(invalid_ownership());
                }
            }
        }
        for employee in roster.iter() {
            let id = employee.employee_definition_id.as_str();
            if !catalog.contains(id) || !owned.insert(id) {
                return Err(invalid_ownership());
            }
        }
        if owned != catalog {
            return Err(invalid_ownership());
        }
        Ok(())
    }
}

fn invalid_ownership() -> GameError {
    GameError::InvalidStaticData(
        "recruitment queue/session/roster ownership is not exact".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::employee::{Employee, EmployeeInitialLoadout, EmployeeProfile};
    use uuid::Uuid;

    fn definitions() -> Vec<EmployeeDefinition> {
        (0..6)
            .map(|index| EmployeeDefinition {
                employee_definition_id: format!("employee_{index}"),
                name: format!("Employee {index}"),
                role: crate::game::employee::EmployeeRole::Dealer,
                starter_eligible: index < 3,
                initial_loadout: EmployeeInitialLoadout::default(),
                profile: EmployeeProfile::default(),
            })
            .collect()
    }

    #[test]
    fn initialization_is_seeded_complete_and_excludes_selected_roster_definitions() {
        let definitions = definitions();
        let selected = vec!["employee_0".to_string(), "employee_2".to_string()];
        let first = RecruitmentState::initialize(17, &definitions, &selected).unwrap();
        let second = RecruitmentState::initialize(17, &definitions, &selected).unwrap();
        let different = RecruitmentState::initialize(18, &definitions, &selected).unwrap();
        let first_ids = first.queue_definition_ids().collect::<Vec<_>>();

        assert_eq!(first, second);
        assert_ne!(first, different);
        assert_eq!(first_ids.len(), 4);
        assert!(!first_ids.contains(&"employee_0"));
        assert!(!first_ids.contains(&"employee_2"));
    }

    #[test]
    fn session_is_fixed_and_recruitment_returns_unselected_candidates_to_tail() {
        let definitions = definitions();
        let selected = vec!["employee_0".to_string()];
        let mut state = RecruitmentState::initialize(17, &definitions, &selected).unwrap();
        let original = state
            .queue_definition_ids()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let node_id = MapNodeId::new(Uuid::from_u128(1));
        let offered = state
            .begin_session(node_id, 3)
            .unwrap()
            .employee_definition_ids
            .clone();
        assert_eq!(
            state
                .begin_session(node_id, 3)
                .unwrap()
                .employee_definition_ids,
            offered
        );

        state.recruit_from_active_session(&offered[1]).unwrap();
        let remaining = state.queue_definition_ids().collect::<Vec<_>>();
        let expected = original[3..]
            .iter()
            .chain([&offered[0], &offered[2]])
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(remaining, expected);
    }

    #[test]
    fn exact_ownership_rejects_duplicate_or_missing_definition() {
        let definitions = definitions();
        let selected = vec!["employee_0".to_string()];
        let mut roster = EmployeeRoster::new();
        roster.add(Employee::from_definition(
            Uuid::from_u128(7),
            &definitions[0],
        ));
        let mut state = RecruitmentState::initialize(17, &definitions, &selected).unwrap();
        state
            .validate_exact_ownership(&definitions, &roster)
            .unwrap();

        state.queue.push_back("employee_0".to_string());
        assert!(state
            .validate_exact_ownership(&definitions, &roster)
            .is_err());
    }
}
